# Git Commit Helper - AI Coding Guidelines

## Project Overview
This is a Rust CLI tool that generates intelligent git commit messages using multiple AI services. It supports bilingual output (Chinese/English), integrates with GitHub/Gerrit for code reviews, and provides automatic commit message translation and enhancement.

## Architecture Patterns

### Core Components
- **CLI Layer** (`main.rs`): Command parsing and orchestration using clap
- **AI Service Layer** (`ai_service.rs`): Abstracted AI provider implementations with async traits
- **Configuration** (`config.rs`): Service management and user preferences
- **Git Integration** (`git.rs`): Hook processing and commit message handling
- **Platform Integrations**: GitHub (`github.rs`), Gerrit (`gerrit.rs`) for remote reviews

### Key Design Patterns
- **Async Service Abstraction**: All AI services implement `AiService` trait with `chat()` and `translate()` methods
- **Fallback Mechanism**: `ai_service::translate_with_fallback()` tries multiple services on failure
- **Progress Reporting**: Long operations use `terminal_format::print_progress()` for user feedback
- **Configuration-Driven**: Service selection and behavior controlled via `Config` struct

## Development Workflow

### Building
```bash
cargo build --release  # Production build
cargo build           # Debug build
```

### Testing
```bash
cargo test           # Run all tests
cargo test --lib     # Library tests only
```

### Installation
```bash
./install.sh         # Quick install from source
git-commit-helper install  # Install git hooks in current repo
```

## Code Patterns

### AI Service Implementation
```rust
#[async_trait]
impl AiService for MyTranslator {
    async fn chat(&self, system_prompt: &str, user_content: &str) -> Result<String> {
        // Implementation with proper error handling
    }
}
```

### Configuration Management
```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default_service: AIService,
    pub services: Vec<AIServiceConfig>,
    // ... other fields
}
```

### Error Handling
- Use `anyhow::Result<()>` for fallible operations
- Prefer `?` operator for error propagation
- Log errors with `log::{debug, info, warn, error}`

### Async Operations
- All AI calls are async with `tokio::main`
- Use `reqwest` for HTTP requests with timeout configuration
- Handle network failures gracefully with retries

## Commit Message Conventions

### Structure
```
<type>(<scope>): <english title>

<detailed english description>

<chinese title>

<chinese description>

Fixes: #123
PMS: BUG-456
```

### Types
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Testing
- `chore`: Maintenance

### Issue Linking
- GitHub: `https://github.com/owner/repo/issues/123` → `Fixes: owner/repo#123`
- PMS: `https://pms.uniontech.com/bug-view-123.html` → `PMS: BUG-123`
- Local: `123` → `Fixes: #123`

## Testing Patterns

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_my_function() {
        // Test implementation
    }
}
```

### CLI Testing
Uses `assert_cmd` for command-line interface testing in `dev-dependencies`.

## Packaging

### Distribution Formats
- **Arch Linux**: `PKGBUILD` with makepkg
- **Debian/Ubuntu**: `debian/` directory with debuild
- **Fedora**: RPM packaging
- **Shell Completions**: bash, zsh, fish in `completions/`

### Build Dependencies
- Rust 1.70+ (stable toolchain)
- Cargo for dependency management
- Git for version control

## Key Files to Reference

### Core Logic
- `src/main.rs`: CLI command structure and main flow
- `src/ai_service.rs`: AI service abstractions and implementations
- `src/commit.rs`: Commit message parsing and generation logic
- `src/config.rs`: Configuration structures and file handling

### Integration Points
- `src/git.rs`: Git hook integration and commit processing
- `src/github.rs`: GitHub API interactions
- `src/gerrit.rs`: Gerrit API interactions

### Utilities
- `src/terminal_format.rs`: Terminal output styling and progress
- `install.sh`: Installation script with dependency checking
- `completions/`: Shell completion files

## Common Tasks

### Adding New AI Service
1. Add variant to `AIService` enum in `config.rs`
2. Implement `AiService` trait in `ai_service.rs`
3. Add service creation logic in `ai_service::create_translator()`
4. Update CLI selection menu in `main.rs`

### Adding New Commit Type
1. Update commit type validation in `commit.rs`
2. Add to documentation in `README.md`
3. Update shell completions if needed

### Platform Integration
1. Create new module (e.g., `myplatform.rs`)
2. Implement review functions following `github.rs`/`gerrit.rs` patterns
3. Add URL pattern matching in `review.rs`
4. Update CLI help text

## Quality Assurance

### Code Review Checklist
- [ ] Async error handling with proper `Result` types
- [ ] Configuration validation and fallbacks
- [ ] Progress reporting for user-facing operations
- [ ] Bilingual output support where applicable
- [ ] Unit tests for new functionality
- [ ] Documentation updates for user-facing changes

### Performance Considerations
- AI requests use configurable timeouts (default 20s)
- Progress reporting prevents perceived hangs
- Fallback mechanism ensures reliability
- Minimal dependencies for fast compilation</content>
<parameter name="filePath">/home/zccrs/projects/git-commit-helper/.github/copilot-instructions.md
