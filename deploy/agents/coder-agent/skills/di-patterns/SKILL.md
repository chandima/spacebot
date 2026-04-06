---
name: di-patterns
description: Use this skill when designing interfaces and module boundaries. Provides dependency injection patterns for testable architecture in Rust, TypeScript, and Python. Includes mock design guidance.
---

# Dependency Injection Patterns

Design for testability. Every external dependency should be injectable so tests can substitute mocks.

## Core Principles

1. **Define interfaces at module boundaries.** The consuming module defines the trait/interface, not the provider.
2. **Accept dependencies as parameters.** Constructor injection over service locators over global state.
3. **Keep side effects at the edges.** Core logic should be pure functions that receive data and return data.
4. **Mock at the boundary, not inside.** Test real logic with fake boundaries, not fake logic with real boundaries.

## Rust Patterns

### Trait-Based DI

```rust
// Define the interface in the consuming module
pub trait UserStore: Send + Sync {
    async fn get_user(&self, id: &str) -> Result<User, StoreError>;
    async fn save_user(&self, user: &User) -> Result<(), StoreError>;
}

// Production implementation
pub struct SqliteUserStore { pool: SqlitePool }

impl UserStore for SqliteUserStore {
    async fn get_user(&self, id: &str) -> Result<User, StoreError> { /* ... */ }
    async fn save_user(&self, user: &User) -> Result<(), StoreError> { /* ... */ }
}

// Consumer accepts the trait
pub struct UserService<S: UserStore> {
    store: S,
}

impl<S: UserStore> UserService<S> {
    pub fn new(store: S) -> Self { Self { store } }
}
```

### Mock Implementation

```rust
#[cfg(test)]
mod tests {
    struct MockUserStore {
        users: std::sync::Mutex<HashMap<String, User>>,
    }

    impl UserStore for MockUserStore {
        async fn get_user(&self, id: &str) -> Result<User, StoreError> {
            self.users.lock().unwrap().get(id).cloned()
                .ok_or(StoreError::NotFound)
        }
        async fn save_user(&self, user: &User) -> Result<(), StoreError> {
            self.users.lock().unwrap().insert(user.id.clone(), user.clone());
            Ok(())
        }
    }
}
```

## TypeScript Patterns

### Interface-Based DI

```typescript
// Interface defined by consumer
interface UserStore {
  getUser(id: string): Promise<User>;
  saveUser(user: User): Promise<void>;
}

// Production implementation
class PostgresUserStore implements UserStore {
  constructor(private pool: Pool) {}
  async getUser(id: string): Promise<User> { /* ... */ }
  async saveUser(user: User): Promise<void> { /* ... */ }
}

// Consumer accepts the interface
class UserService {
  constructor(private store: UserStore) {}
}

// Test mock
class MockUserStore implements UserStore {
  private users = new Map<string, User>();
  async getUser(id: string): Promise<User> {
    const user = this.users.get(id);
    if (!user) throw new NotFoundError();
    return user;
  }
  async saveUser(user: User): Promise<void> {
    this.users.set(user.id, user);
  }
}
```

## Python Patterns

### Protocol-Based DI

```python
from typing import Protocol

class UserStore(Protocol):
    async def get_user(self, id: str) -> User: ...
    async def save_user(self, user: User) -> None: ...

class UserService:
    def __init__(self, store: UserStore):
        self.store = store

# Test mock
class MockUserStore:
    def __init__(self):
        self.users: dict[str, User] = {}

    async def get_user(self, id: str) -> User:
        if id not in self.users:
            raise NotFoundError()
        return self.users[id]

    async def save_user(self, user: User) -> None:
        self.users[user.id] = user
```

## Mock Design Guidelines

- **Mocks should be simple.** If your mock is complex, the interface is too complex.
- **Mocks should be in-memory.** HashMap/Map/dict backends, not test databases.
- **Mocks should be deterministic.** No random behavior, no timing dependencies.
- **Test behavior, not calls.** Assert on outcomes, not "was this method called."
- **One mock per boundary.** Don't mock internal components, only external dependencies.
