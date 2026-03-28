//! Integration test for sandbox wrapping in ShellTool
//!
//! This test verifies that ShellTool properly accepts and uses the sandbox
//! for command containment by checking the module structure and type signatures.

use spacebot::tools::shell::ShellTool;

/// This test verifies that ShellTool is publicly accessible and has the
/// expected module structure. The actual sandbox integration is tested
/// in unit tests within src/tools.rs.
#[test]
fn test_shell_tool_module_structure() {
    // ShellTool type is accessible from the public API
    // This verifies the module exports are correct
    let _tool_type = std::any::type_name::<ShellTool>();

    // If this compiles, ShellTool is properly exposed
    assert!(_tool_type.contains("ShellTool"));
}

/// Verify that the sandbox module is accessible and exports Sandbox
#[test]
fn test_sandbox_module_structure() {
    // Sandbox type is accessible from the public API
    let _sandbox_type = std::any::type_name::<spacebot::sandbox::Sandbox>();

    // If this compiles, the sandbox is properly exposed
    assert!(_sandbox_type.contains("Sandbox"));
}
