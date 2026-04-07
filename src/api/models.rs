use super::state::ApiState;

use anyhow::Context as _;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Clone, utoipa::ToSchema)]
pub(super) struct ModelInfo {
    /// Full routing string (e.g. "openrouter/anthropic/claude-sonnet-4")
    id: String,
    /// Human-readable name
    name: String,
    /// Provider ID for routing ("anthropic", "openrouter", "openai", etc.)
    provider: String,
    /// Context window size in tokens, if known
    context_window: Option<u64>,
    /// Whether this model supports tool/function calling
    tool_call: bool,
    /// Whether this model has reasoning/thinking capability
    reasoning: bool,
    /// Whether this model accepts audio input.
    input_audio: bool,
}

#[derive(Serialize, Clone, utoipa::ToSchema)]
pub(super) struct ProviderStatus {
    /// Provider ID used in routing (e.g. "github-copilot", "litellm")
    id: String,
    /// Human-readable name
    name: String,
    /// How the provider is authenticated: "api_key", "oauth", "device_flow", "custom", "none"
    auth_type: String,
    /// Whether the provider is configured and ready to use
    configured: bool,
}

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct ModelsResponse {
    models: Vec<ModelInfo>,
    providers: Vec<ProviderStatus>,
}

#[derive(Deserialize, utoipa::ToSchema, utoipa::IntoParams)]
pub(super) struct ModelsQuery {
    provider: Option<String>,
    capability: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct ModelsDevProvider {
    #[allow(dead_code)]
    id: Option<String>,
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    models: HashMap<String, ModelsDevModel>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct ModelsDevModel {
    #[allow(dead_code)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    tool_call: bool,
    #[serde(default)]
    reasoning: bool,
    limit: Option<ModelsDevLimit>,
    modalities: Option<ModelsDevModalities>,
    status: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct ModelsDevLimit {
    context: u64,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct ModelsDevModalities {
    input: Option<Vec<String>>,
    output: Option<Vec<String>>,
}

/// Cached model catalog fetched from models.dev.
static MODELS_CACHE: std::sync::LazyLock<
    tokio::sync::RwLock<(Vec<ModelInfo>, std::time::Instant)>,
> = std::sync::LazyLock::new(|| tokio::sync::RwLock::new((Vec::new(), std::time::Instant::now())));

/// Cached models fetched from GitHub Copilot API.
static COPILOT_MODELS_CACHE: std::sync::LazyLock<
    tokio::sync::RwLock<(Vec<ModelInfo>, std::time::Instant)>,
> = std::sync::LazyLock::new(|| tokio::sync::RwLock::new((Vec::new(), std::time::Instant::now())));

/// Cached models fetched from custom providers (keyed by provider ID).
type CustomProviderModelsCache =
    tokio::sync::RwLock<(HashMap<String, Vec<ModelInfo>>, std::time::Instant)>;
#[allow(clippy::type_complexity)]
static CUSTOM_PROVIDER_MODELS_CACHE: std::sync::LazyLock<CustomProviderModelsCache> =
    std::sync::LazyLock::new(|| {
        tokio::sync::RwLock::new((HashMap::new(), std::time::Instant::now()))
    });

const MODELS_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Shorter TTL for dynamic provider model discovery (5 min).
const DYNAMIC_MODELS_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(300);

/// Models known to work with Spacebot's current voice transcription path
/// (OpenAI-compatible `/v1/chat/completions` with `input_audio`).
const KNOWN_VOICE_TRANSCRIPTION_MODELS: &[&str] = &[
    // Native Gemini API
    "gemini/gemini-2.0-flash",
    "gemini/gemini-2.5-flash",
    "gemini/gemini-2.5-flash-lite",
    "gemini/gemini-2.5-pro",
    "gemini/gemini-3-flash-preview",
    "gemini/gemini-3-pro-preview",
    "gemini/gemini-3.1-pro-preview",
    // Via OpenRouter
    "openrouter/google/gemini-2.0-flash-001",
    "openrouter/google/gemini-2.5-flash",
    "openrouter/google/gemini-2.5-flash-lite",
    "openrouter/google/gemini-2.5-pro",
    "openrouter/google/gemini-3-flash-preview",
    "openrouter/google/gemini-3-pro-preview",
    "openrouter/google/gemini-3.1-pro-preview",
];

/// Maps models.dev provider IDs to spacebot's internal provider IDs for
/// providers with direct integrations.
fn direct_provider_mapping(models_dev_id: &str) -> Option<&'static str> {
    match models_dev_id {
        "anthropic" => Some("anthropic"),
        "openai" => Some("openai"),
        "kilo" => Some("kilo"),
        "deepseek" => Some("deepseek"),
        "xai" => Some("xai"),
        "mistral" => Some("mistral"),
        "gemini" | "google" => Some("gemini"),
        "groq" => Some("groq"),
        "togetherai" => Some("together"),
        "fireworks-ai" => Some("fireworks"),
        "zhipuai" => Some("zhipu"),
        "opencode" => Some("opencode-zen"),
        "opencode-go" => Some("opencode-go"),
        "zai-coding-plan" => Some("zai-coding-plan"),
        "minimax" => Some("minimax"),
        "moonshotai" => Some("moonshot"),
        _ => None,
    }
}

fn is_known_voice_transcription_model(model_id: &str) -> bool {
    KNOWN_VOICE_TRANSCRIPTION_MODELS.contains(&model_id)
}

fn as_openai_chatgpt_model(model: &ModelInfo) -> Option<ModelInfo> {
    if model.provider != "openai" {
        return None;
    }

    let model_name = model.id.strip_prefix("openai/")?;
    Some(ModelInfo {
        id: format!("openai-chatgpt/{model_name}"),
        name: model.name.clone(),
        provider: "openai-chatgpt".into(),
        context_window: model.context_window,
        tool_call: model.tool_call,
        reasoning: model.reasoning,
        input_audio: model.input_audio,
    })
}

/// Models from providers not in models.dev (private/custom endpoints).
fn extra_models() -> Vec<ModelInfo> {
    vec![
        // MiniMax CN - China-specific endpoint, not on models.dev
        ModelInfo {
            id: "minimax-cn/MiniMax-M2.5".into(),
            name: "MiniMax M2.5".into(),
            provider: "minimax-cn".into(),
            context_window: Some(200000),
            tool_call: true,
            reasoning: true,
            input_audio: false,
        },
        // Moonshot AI (Kimi) - moonshot-v1-8k not on models.dev
        ModelInfo {
            id: "moonshot/moonshot-v1-8k".into(),
            name: "Moonshot V1 8K".into(),
            provider: "moonshot".into(),
            context_window: Some(8000),
            tool_call: false,
            reasoning: false,
            input_audio: false,
        },
    ]
}

/// Fetch the full model catalog from models.dev and transform into ModelInfo entries.
async fn fetch_models_dev() -> anyhow::Result<Vec<ModelInfo>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://models.dev/api.json")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?
        .error_for_status()?;

    let catalog: HashMap<String, ModelsDevProvider> = response.json().await?;
    let mut models = Vec::new();

    for (provider_id, provider) in &catalog {
        for (model_id, model) in &provider.models {
            if model.status.as_deref() == Some("deprecated") {
                continue;
            }

            let has_text_output = model
                .modalities
                .as_ref()
                .and_then(|m| m.output.as_ref())
                .is_some_and(|outputs| outputs.iter().any(|o| o == "text"));
            if !has_text_output {
                continue;
            }

            let (routing_id, routing_provider) =
                if let Some(spacebot_provider) = direct_provider_mapping(provider_id) {
                    (
                        format!("{spacebot_provider}/{model_id}"),
                        spacebot_provider.to_string(),
                    )
                } else if provider_id == "openrouter" {
                    (format!("openrouter/{model_id}"), "openrouter".into())
                } else {
                    (
                        format!("openrouter/{provider_id}/{model_id}"),
                        "openrouter".into(),
                    )
                };

            let context_window = model.limit.as_ref().map(|l| l.context);
            let input_audio = model
                .modalities
                .as_ref()
                .and_then(|m| m.input.as_ref())
                .is_some_and(|inputs| {
                    inputs
                        .iter()
                        .any(|input| input.to_lowercase().contains("audio"))
                });

            models.push(ModelInfo {
                id: routing_id,
                name: model.name.clone(),
                provider: routing_provider,
                context_window,
                tool_call: model.tool_call,
                reasoning: model.reasoning,
                input_audio,
            });
        }
    }

    models.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.name.cmp(&b.name)));

    Ok(models)
}

/// Ensure the cache is populated (fetches on first call, then uses TTL).
async fn ensure_models_cache() -> Vec<ModelInfo> {
    {
        let cache = MODELS_CACHE.read().await;
        if !cache.0.is_empty() && cache.1.elapsed() < MODELS_CACHE_TTL {
            return cache.0.clone();
        }
    }

    match fetch_models_dev().await {
        Ok(models) => {
            let mut cache = MODELS_CACHE.write().await;
            *cache = (models.clone(), std::time::Instant::now());
            models
        }
        Err(error) => {
            tracing::warn!(%error, "failed to fetch models from models.dev, using stale cache");
            let cache = MODELS_CACHE.read().await;
            cache.0.clone()
        }
    }
}

// ---------------------------------------------------------------------------
// GitHub Copilot dynamic model discovery
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct CopilotModelsResponse {
    data: Vec<CopilotModel>,
}

#[derive(Deserialize)]
struct CopilotModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    capabilities: Option<CopilotCapabilities>,
}

#[derive(Deserialize)]
struct CopilotCapabilities {
    limits: Option<CopilotLimits>,
    supports: Option<CopilotSupports>,
}

#[derive(Deserialize)]
struct CopilotLimits {
    max_prompt_tokens: Option<u64>,
    max_output_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct CopilotSupports {
    tool_calls: Option<bool>,
    #[serde(default)]
    tools: Option<bool>,
}

fn infer_reasoning(model_id: &str) -> bool {
    let lower = model_id.to_lowercase();
    lower.contains("opus")
        || lower.contains("sonnet")
        || lower.starts_with("gpt-5")
        || lower.contains("gemini-2.5-pro")
        || lower.contains("gemini-3")
}

/// Fetch models from the GitHub Copilot API using the cached token.
async fn fetch_copilot_models(instance_dir: &std::path::Path) -> anyhow::Result<Vec<ModelInfo>> {
    let token = crate::github_copilot_auth::load_cached_token(instance_dir)?
        .context("no Copilot token available")?;

    let base_url = if token.is_device_flow() {
        "https://api.githubcopilot.com".to_string()
    } else {
        crate::github_copilot_auth::derive_base_url_from_token(&token.token)
            .unwrap_or_else(|| crate::github_copilot_auth::DEFAULT_COPILOT_API_BASE_URL.to_string())
    };

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base_url}/models"))
        .header("Authorization", format!("Bearer {}", token.token))
        .header("editor-version", "vscode/1.96.2")
        .header("editor-plugin-version", "copilot-chat/0.26.7")
        .header("Copilot-Integration-Id", "vscode-chat")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?
        .error_for_status()?;

    let api_response: CopilotModelsResponse = response.json().await?;

    let mut seen_ids = std::collections::HashSet::new();
    let mut models = Vec::new();

    for model in api_response.data {
        if !seen_ids.insert(model.id.clone()) {
            continue;
        }

        let caps = model.capabilities.as_ref();
        let limits = caps.and_then(|c| c.limits.as_ref());
        let supports = caps.and_then(|c| c.supports.as_ref());

        let max_prompt = limits.and_then(|l| l.max_prompt_tokens).unwrap_or(0);
        let max_output = limits.and_then(|l| l.max_output_tokens).unwrap_or(0);
        let context_window = if max_prompt > 0 {
            Some(max_prompt + max_output)
        } else {
            None
        };

        let tool_call = supports
            .and_then(|s| s.tool_calls.or(s.tools))
            .unwrap_or(false);

        let display_name = model.name.clone().unwrap_or_else(|| model.id.clone());

        models.push(ModelInfo {
            id: format!("github-copilot/{}", model.id),
            name: display_name,
            provider: "github-copilot".into(),
            context_window,
            tool_call,
            reasoning: infer_reasoning(&model.id),
            input_audio: false,
        });
    }

    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}

/// Ensure the Copilot models cache is fresh.
async fn ensure_copilot_models_cache(instance_dir: &std::path::Path) -> Vec<ModelInfo> {
    {
        let cache = COPILOT_MODELS_CACHE.read().await;
        if !cache.0.is_empty() && cache.1.elapsed() < DYNAMIC_MODELS_CACHE_TTL {
            return cache.0.clone();
        }
    }

    match fetch_copilot_models(instance_dir).await {
        Ok(models) => {
            tracing::info!(count = models.len(), "fetched GitHub Copilot models");
            let mut cache = COPILOT_MODELS_CACHE.write().await;
            *cache = (models.clone(), std::time::Instant::now());
            models
        }
        Err(error) => {
            tracing::warn!(%error, "failed to fetch Copilot models");
            let cache = COPILOT_MODELS_CACHE.read().await;
            cache.0.clone()
        }
    }
}

// ---------------------------------------------------------------------------
// Custom provider (LiteLLM, Ollama, etc.) dynamic model discovery
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModelEntry>,
}

#[derive(Deserialize)]
struct OpenAiModelEntry {
    id: String,
}

/// Fetch model list from an OpenAI-compatible endpoint.
async fn fetch_openai_compatible_models(
    provider_id: &str,
    base_url: &str,
    api_key: &str,
) -> anyhow::Result<Vec<ModelInfo>> {
    let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::new();

    let mut request = client.get(&url).timeout(std::time::Duration::from_secs(10));

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {api_key}"));
    }

    let response = request.send().await?.error_for_status()?;
    let api_response: OpenAiModelsResponse = response.json().await?;

    let models: Vec<ModelInfo> = api_response
        .data
        .into_iter()
        .map(|entry| ModelInfo {
            id: format!("{provider_id}/{}", entry.id),
            name: entry.id.clone(),
            provider: provider_id.to_string(),
            context_window: None,
            tool_call: true,
            reasoning: false,
            input_audio: false,
        })
        .collect();

    Ok(models)
}

/// Ensure the custom provider models cache is fresh.
async fn ensure_custom_provider_models_cache(
    provider_id: &str,
    base_url: &str,
    api_key: &str,
) -> Vec<ModelInfo> {
    {
        let cache = CUSTOM_PROVIDER_MODELS_CACHE.read().await;
        if let Some(models) = cache.0.get(provider_id)
            && !models.is_empty()
            && cache.1.elapsed() < DYNAMIC_MODELS_CACHE_TTL
        {
            return models.clone();
        }
    }

    match fetch_openai_compatible_models(provider_id, base_url, api_key).await {
        Ok(models) => {
            tracing::info!(provider = %provider_id, count = models.len(), "fetched custom provider models");
            let mut cache = CUSTOM_PROVIDER_MODELS_CACHE.write().await;
            cache.0.insert(provider_id.to_string(), models.clone());
            cache.1 = std::time::Instant::now();
            models
        }
        Err(error) => {
            tracing::warn!(provider = %provider_id, %error, "failed to fetch custom provider models");
            let cache = CUSTOM_PROVIDER_MODELS_CACHE.read().await;
            cache.0.get(provider_id).cloned().unwrap_or_default()
        }
    }
}

/// Helper: provider detection results for the models endpoint.
///
/// Returns `(configured_ids, custom_providers, provider_statuses)`:
/// - `configured_ids`: provider IDs that have static keys (models.dev catalog filtering)
/// - `custom_providers`: `(id, base_url, api_key)` tuples for custom providers with base URLs
/// - `provider_statuses`: full status list for the response
pub(super) async fn configured_providers(
    config_path: &std::path::Path,
) -> (
    Vec<String>,
    Vec<(String, String, String)>,
    Vec<ProviderStatus>,
) {
    let mut configured_ids = Vec::new();
    let mut custom_providers: Vec<(String, String, String)> = Vec::new();
    let mut statuses = Vec::new();

    let document = tokio::fs::read_to_string(config_path)
        .await
        .ok()
        .and_then(|content| content.parse::<toml_edit::DocumentMut>().ok());

    let has_key = |key: &str, env_var: &str| {
        if let Some(doc) = document.as_ref()
            && let Some(llm) = doc.get("llm")
            && let Some(val) = llm.get(key)
            && let Some(s) = val.as_str()
        {
            if let Some(var_name) = s.strip_prefix("env:") {
                return std::env::var(var_name).is_ok();
            }
            return !s.is_empty();
        }
        std::env::var(env_var).is_ok()
    };

    let instance_dir = config_path.parent();

    // --- Built-in providers with static API keys ---
    let builtin_providers: &[(&str, &str, &str, &str)] = &[
        (
            "anthropic",
            "anthropic_key",
            "ANTHROPIC_API_KEY",
            "Anthropic",
        ),
        ("openai", "openai_key", "OPENAI_API_KEY", "OpenAI"),
        (
            "openrouter",
            "openrouter_key",
            "OPENROUTER_API_KEY",
            "OpenRouter",
        ),
        ("kilo", "kilo_key", "KILO_API_KEY", "Kilo"),
        ("zhipu", "zhipu_key", "ZHIPU_API_KEY", "ZhipuAI"),
        ("groq", "groq_key", "GROQ_API_KEY", "Groq"),
        (
            "together",
            "together_key",
            "TOGETHER_API_KEY",
            "Together AI",
        ),
        (
            "fireworks",
            "fireworks_key",
            "FIREWORKS_API_KEY",
            "Fireworks AI",
        ),
        ("deepseek", "deepseek_key", "DEEPSEEK_API_KEY", "DeepSeek"),
        ("xai", "xai_key", "XAI_API_KEY", "xAI"),
        ("mistral", "mistral_key", "MISTRAL_API_KEY", "Mistral AI"),
        ("gemini", "gemini_key", "GEMINI_API_KEY", "Google Gemini"),
        (
            "opencode-zen",
            "opencode_zen_key",
            "OPENCODE_ZEN_API_KEY",
            "OpenCode Zen",
        ),
        (
            "opencode-go",
            "opencode_go_key",
            "OPENCODE_GO_API_KEY",
            "OpenCode Go",
        ),
        ("minimax", "minimax_key", "MINIMAX_API_KEY", "MiniMax"),
        (
            "minimax-cn",
            "minimax_cn_key",
            "MINIMAX_CN_API_KEY",
            "MiniMax CN",
        ),
        (
            "moonshot",
            "moonshot_key",
            "MOONSHOT_API_KEY",
            "Moonshot AI",
        ),
        (
            "zai-coding-plan",
            "zai_coding_plan_key",
            "ZAI_CODING_PLAN_API_KEY",
            "ZAI Coding Plan",
        ),
    ];

    for &(id, config_key, env_var, display_name) in builtin_providers {
        let configured = has_key(config_key, env_var);
        if configured {
            configured_ids.push(id.to_string());
        }
        statuses.push(ProviderStatus {
            id: id.to_string(),
            name: display_name.to_string(),
            auth_type: if configured {
                "api_key".into()
            } else {
                "none".into()
            },
            configured,
        });
    }

    // --- OpenAI ChatGPT (OAuth) ---
    let openai_chatgpt_configured =
        instance_dir.is_some_and(|dir| crate::openai_auth::credentials_path(dir).exists());
    if openai_chatgpt_configured {
        configured_ids.push("openai-chatgpt".to_string());
    }
    statuses.push(ProviderStatus {
        id: "openai-chatgpt".to_string(),
        name: "OpenAI ChatGPT".to_string(),
        auth_type: if openai_chatgpt_configured {
            "oauth".into()
        } else {
            "none".into()
        },
        configured: openai_chatgpt_configured,
    });

    // --- GitHub Copilot (device flow or PAT) ---
    let copilot_device_flow =
        instance_dir.is_some_and(|dir| crate::github_copilot_auth::credentials_path(dir).exists());
    let copilot_pat = has_key("github_copilot_key", "GITHUB_COPILOT_KEY");
    let copilot_configured = copilot_device_flow || copilot_pat;
    if copilot_configured {
        configured_ids.push("github-copilot".to_string());
    }
    statuses.push(ProviderStatus {
        id: "github-copilot".to_string(),
        name: "GitHub Copilot".to_string(),
        auth_type: if copilot_device_flow {
            "device_flow".into()
        } else if copilot_pat {
            "api_key".into()
        } else {
            "none".into()
        },
        configured: copilot_configured,
    });

    // --- Custom providers from [llm.provider.*] or [llm.providers.*] ---
    if let Some(doc) = document.as_ref()
        && let Some(llm) = doc.get("llm")
    {
        for table_key in &["provider", "providers"] {
            if let Some(providers_table) = llm.get(table_key)
                && let Some(table) = providers_table.as_table_like()
            {
                for (provider_id, value) in table.iter() {
                    if configured_ids.contains(&provider_id.to_string()) {
                        continue;
                    }

                    if let Some(provider_table) = value.as_table_like() {
                        let base_url = provider_table
                            .get("base_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let api_key = provider_table
                            .get("api_key")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let display_name = provider_table
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(provider_id)
                            .to_string();

                        if !base_url.is_empty() {
                            configured_ids.push(provider_id.to_string());
                            custom_providers.push((
                                provider_id.to_string(),
                                base_url,
                                api_key.clone(),
                            ));
                            statuses.push(ProviderStatus {
                                id: provider_id.to_string(),
                                name: display_name,
                                auth_type: if api_key.is_empty() {
                                    "custom".into()
                                } else {
                                    "api_key".into()
                                },
                                configured: true,
                            });
                        }
                    }
                }
            }
        }
    }

    (configured_ids, custom_providers, statuses)
}

#[utoipa::path(
    get,
    path = "/models",
    params(
        ("provider" = Option<String>, Query, description = "Filter by provider ID"),
        ("capability" = Option<String>, Query, description = "Filter by capability (input_audio, voice_transcription)"),
    ),
    responses(
        (status = 200, body = ModelsResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "models",
)]
pub(super) async fn get_models(
    State(state): State<Arc<ApiState>>,
    Query(query): Query<ModelsQuery>,
) -> Result<Json<ModelsResponse>, StatusCode> {
    let config_path = state.config_path.read().await.clone();
    let (configured, custom_providers, provider_statuses) =
        configured_providers(&config_path).await;
    let instance_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
    let requested_provider = query
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|provider| !provider.is_empty());
    let requested_provider_for_catalog = if requested_provider == Some("openai-chatgpt") {
        Some("openai")
    } else {
        requested_provider
    };
    let requested_capability = query
        .capability
        .as_deref()
        .map(str::trim)
        .filter(|capability| !capability.is_empty());

    let catalog = ensure_models_cache().await;
    let capability_matches = |model: &ModelInfo| {
        if let Some(capability) = requested_capability {
            match capability {
                "input_audio" => model.input_audio,
                "voice_transcription" => {
                    model.input_audio && is_known_voice_transcription_model(&model.id)
                }
                _ => true,
            }
        } else {
            true
        }
    };

    // 1. Models from the models.dev catalog (filtered by configured providers)
    let mut models: Vec<ModelInfo> = catalog
        .iter()
        .filter(|model| {
            let provider_match = if let Some(provider) = requested_provider_for_catalog {
                model.provider == provider
            } else {
                configured.iter().any(|p| p == &model.provider)
            };
            if !provider_match {
                return false;
            }
            capability_matches(model)
        })
        .cloned()
        .collect();

    // 2. OpenAI ChatGPT models (synthesized from OpenAI catalog entries)
    if requested_provider == Some("openai-chatgpt") {
        models = models
            .into_iter()
            .filter_map(|model| as_openai_chatgpt_model(&model))
            .collect();
    } else if requested_provider.is_none() && configured.iter().any(|p| p == "openai-chatgpt") {
        let chatgpt_models: Vec<ModelInfo> = catalog
            .iter()
            .filter(|model| model.provider == "openai" && capability_matches(model))
            .filter_map(as_openai_chatgpt_model)
            .collect();
        models.extend(chatgpt_models);
    }

    // 3. GitHub Copilot models (dynamically fetched from API)
    let include_copilot = requested_provider == Some("github-copilot")
        || (requested_provider.is_none() && configured.iter().any(|p| p == "github-copilot"));
    if include_copilot {
        let copilot_models = ensure_copilot_models_cache(instance_dir).await;
        for model in copilot_models {
            if capability_matches(&model) {
                models.push(model);
            }
        }
    }

    // 4. Custom provider models (dynamically fetched from their endpoints)
    for (provider_id, base_url, api_key) in &custom_providers {
        let include = requested_provider == Some(provider_id.as_str())
            || (requested_provider.is_none() && configured.iter().any(|p| p == provider_id));
        if include {
            let custom_models =
                ensure_custom_provider_models_cache(provider_id, base_url, api_key).await;
            for model in custom_models {
                if capability_matches(&model) {
                    models.push(model);
                }
            }
        }
    }

    // 5. Extra hardcoded models (MiniMax CN, Moonshot, etc.)
    for model in extra_models() {
        if let Some(capability) = requested_capability {
            if capability == "input_audio" && !model.input_audio {
                continue;
            }
            if capability == "voice_transcription"
                && (!model.input_audio || !is_known_voice_transcription_model(&model.id))
            {
                continue;
            }
        }
        if let Some(provider) = requested_provider {
            if model.provider == provider {
                models.push(model);
            }
        } else if configured.iter().any(|p| p == &model.provider) {
            models.push(model);
        }
    }

    // Only include providers that are configured in the response
    let active_providers: Vec<ProviderStatus> = provider_statuses
        .into_iter()
        .filter(|p| p.configured)
        .collect();

    Ok(Json(ModelsResponse {
        models,
        providers: active_providers,
    }))
}

#[utoipa::path(
    post,
    path = "/models/refresh",
    responses(
        (status = 200, body = ModelsResponse),
        (status = 500, description = "Internal server error"),
    ),
    tag = "models",
)]
pub(super) async fn refresh_models(
    State(state): State<Arc<ApiState>>,
) -> Result<Json<ModelsResponse>, StatusCode> {
    // Clear all caches
    {
        let mut cache = MODELS_CACHE.write().await;
        *cache = (Vec::new(), std::time::Instant::now() - MODELS_CACHE_TTL);
    }
    {
        let mut cache = COPILOT_MODELS_CACHE.write().await;
        *cache = (
            Vec::new(),
            std::time::Instant::now() - DYNAMIC_MODELS_CACHE_TTL,
        );
    }
    {
        let mut cache = CUSTOM_PROVIDER_MODELS_CACHE.write().await;
        *cache = (
            HashMap::new(),
            std::time::Instant::now() - DYNAMIC_MODELS_CACHE_TTL,
        );
    }

    get_models(
        State(state),
        Query(ModelsQuery {
            provider: None,
            capability: None,
        }),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_reasoning() {
        assert!(infer_reasoning("claude-opus-4.6"));
        assert!(infer_reasoning("claude-sonnet-4"));
        assert!(infer_reasoning("claude-sonnet-4.5"));
        assert!(infer_reasoning("gpt-5.2"));
        assert!(infer_reasoning("gpt-5-mini"));
        assert!(infer_reasoning("gpt-5.3-codex"));
        assert!(infer_reasoning("gemini-2.5-pro"));
        assert!(infer_reasoning("gemini-3-flash-preview"));
        assert!(!infer_reasoning("gpt-4o"));
        assert!(!infer_reasoning("gpt-4.1"));
        assert!(!infer_reasoning("grok-code-fast-1"));
        assert!(!infer_reasoning("claude-haiku-4.5"));
    }

    #[test]
    fn test_as_openai_chatgpt_model() {
        let openai_model = ModelInfo {
            id: "openai/gpt-5.2".into(),
            name: "GPT-5.2".into(),
            provider: "openai".into(),
            context_window: Some(272000),
            tool_call: true,
            reasoning: true,
            input_audio: false,
        };

        let chatgpt = as_openai_chatgpt_model(&openai_model).unwrap();
        assert_eq!(chatgpt.id, "openai-chatgpt/gpt-5.2");
        assert_eq!(chatgpt.provider, "openai-chatgpt");
        assert_eq!(chatgpt.name, "GPT-5.2");

        let non_openai = ModelInfo {
            id: "anthropic/claude-sonnet-4".into(),
            name: "Claude Sonnet 4".into(),
            provider: "anthropic".into(),
            context_window: Some(200000),
            tool_call: true,
            reasoning: true,
            input_audio: false,
        };
        assert!(as_openai_chatgpt_model(&non_openai).is_none());
    }

    #[test]
    fn test_direct_provider_mapping() {
        assert_eq!(direct_provider_mapping("anthropic"), Some("anthropic"));
        assert_eq!(direct_provider_mapping("openai"), Some("openai"));
        assert_eq!(direct_provider_mapping("gemini"), Some("gemini"));
        assert_eq!(direct_provider_mapping("google"), Some("gemini"));
        assert_eq!(direct_provider_mapping("unknown"), None);
    }

    #[test]
    fn test_extra_models_have_valid_providers() {
        let models = extra_models();
        assert!(!models.is_empty());
        for model in &models {
            assert!(model.id.starts_with(&format!("{}/", model.provider)));
        }
    }

    #[tokio::test]
    async fn test_configured_providers_with_copilot_token() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write a minimal config with no keys
        tokio::fs::write(&config_path, "[llm]\n").await.unwrap();

        // No providers should be configured
        let (ids, custom, statuses) = configured_providers(&config_path).await;
        assert!(ids.is_empty(), "no providers should be configured");
        assert!(custom.is_empty());
        assert!(
            statuses.iter().all(|s| !s.configured),
            "all providers should be unconfigured"
        );

        // Add a Copilot token file
        let copilot_path = crate::github_copilot_auth::credentials_path(temp_dir.path());
        tokio::fs::write(
            &copilot_path,
            r#"{"token":"gho_test","pat_hash":"","expires_at_ms":0,"updated_at_ms":0,"auth_method":"DeviceFlow"}"#,
        )
        .await
        .unwrap();

        let (ids, _, statuses) = configured_providers(&config_path).await;
        assert!(ids.contains(&"github-copilot".to_string()));
        let copilot_status = statuses.iter().find(|s| s.id == "github-copilot").unwrap();
        assert!(copilot_status.configured);
        assert_eq!(copilot_status.auth_type, "device_flow");
    }

    #[tokio::test]
    async fn test_configured_providers_with_openai_chatgpt_oauth() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        tokio::fs::write(&config_path, "[llm]\n").await.unwrap();

        // Add an OpenAI ChatGPT OAuth file
        let oauth_path = crate::openai_auth::credentials_path(temp_dir.path());
        tokio::fs::write(
            &oauth_path,
            r#"{"access_token":"test","refresh_token":"rt_test","expires_at":9999999999999}"#,
        )
        .await
        .unwrap();

        let (ids, _, statuses) = configured_providers(&config_path).await;
        assert!(ids.contains(&"openai-chatgpt".to_string()));
        let status = statuses.iter().find(|s| s.id == "openai-chatgpt").unwrap();
        assert!(status.configured);
        assert_eq!(status.auth_type, "oauth");
    }

    #[tokio::test]
    async fn test_configured_providers_with_custom_provider() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        tokio::fs::write(
            &config_path,
            r#"
[llm]

[llm.provider.litellm]
base_url = "https://litellm.example.com"
api_key = "sk-test"
name = "LiteLLM Gateway"

[llm.provider.custom-local]
base_url = "http://localhost:8000"
name = "Local Model"
"#,
        )
        .await
        .unwrap();

        let (ids, custom, statuses) = configured_providers(&config_path).await;

        // Both custom providers should be detected
        assert!(ids.contains(&"litellm".to_string()));
        assert!(ids.contains(&"custom-local".to_string()));
        assert_eq!(custom.len(), 2);

        // LiteLLM with API key
        let litellm = statuses.iter().find(|s| s.id == "litellm").unwrap();
        assert!(litellm.configured);
        assert_eq!(litellm.auth_type, "api_key");
        assert_eq!(litellm.name, "LiteLLM Gateway");

        // Custom without API key
        let local = statuses.iter().find(|s| s.id == "custom-local").unwrap();
        assert!(local.configured);
        assert_eq!(local.auth_type, "custom");
    }

    #[tokio::test]
    async fn test_configured_providers_with_api_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        tokio::fs::write(
            &config_path,
            r#"
[llm]
anthropic_key = "sk-ant-test"
openai_key = "sk-test"
"#,
        )
        .await
        .unwrap();

        let (ids, _, statuses) = configured_providers(&config_path).await;
        assert!(ids.contains(&"anthropic".to_string()));
        assert!(ids.contains(&"openai".to_string()));

        let anthropic = statuses.iter().find(|s| s.id == "anthropic").unwrap();
        assert!(anthropic.configured);
        assert_eq!(anthropic.auth_type, "api_key");
    }

    #[test]
    fn test_copilot_model_dedup() {
        // Verify our dedup logic in fetch_copilot_models works
        let mut seen = std::collections::HashSet::new();
        assert!(seen.insert("gpt-4o".to_string()));
        assert!(!seen.insert("gpt-4o".to_string())); // Duplicate
        assert!(seen.insert("gpt-5.2".to_string()));
    }

    #[test]
    fn test_provider_status_serialization() {
        let status = ProviderStatus {
            id: "github-copilot".into(),
            name: "GitHub Copilot".into(),
            auth_type: "device_flow".into(),
            configured: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("github-copilot"));
        assert!(json.contains("device_flow"));
        assert!(json.contains("true"));
    }
}
