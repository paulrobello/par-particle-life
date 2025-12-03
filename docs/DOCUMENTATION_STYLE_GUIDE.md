# Documentation Style Guide

This guide establishes standards and best practices for creating and maintaining documentation in the Par Particle Life project. It ensures consistency, clarity, and professionalism across all technical documentation.

## Table of Contents
- [Quick Reference](#quick-reference)
- [Document Structure](#document-structure)
- [Writing Style](#writing-style)
- [Diagram Standards](#diagram-standards)
- [Color Scheme](#color-scheme)
- [Code Examples](#code-examples)
- [API Documentation](#api-documentation)
- [File Organization](#file-organization)
- [Maintenance](#maintenance)
- [Review Checklist](#review-checklist)

## Quick Reference

### Essential Rules
- **Never use ASCII art** - Always use Mermaid diagrams
- **Always include a TOC** for documents > 500 words
- **Use dark backgrounds** with white text in diagrams
- **Specify language** in all code blocks
- **Test all examples** before documenting
- **Update cross-references** when modifying content
- **Never use line numbers** in file references - They're brittle and hard to maintain
- **Do not store package versions** in documentation - Version numbers are brittle and difficult to maintain

## Document Structure

### Required Elements

Every documentation file MUST include:

1. **Title (H1)**: Clear, descriptive title that immediately identifies the document's purpose
2. **Brief Description**: 1-2 sentence summary immediately after the title
3. **Table of Contents**: Required for documents > 500 words or > 3 main sections
4. **Overview Section**: Context and scope of the document
5. **Main Content**: Logical sections with proper hierarchy
6. **Related Documentation**: Links to relevant resources at the end

### Section Hierarchy

| Level | Usage | Markdown | Example |
|-------|-------|----------|---------|
| H1 | Document title only | `#` | `# API Documentation` |
| H2 | Main sections | `##` | `## Authentication` |
| H3 | Subsections | `###` | `### OAuth Flow` |
| H4 | Minor divisions | `####` | `#### Error Codes` |
| H5+ | Avoid if possible | `#####` | Use lists instead |

## Diagram Standards

### Always Use Mermaid

**NEVER use ASCII art diagrams.** Always use Mermaid for:
- Architecture diagrams
- Flow charts
- Sequence diagrams
- State diagrams
- Entity relationship diagrams

## Color Scheme

### High-Contrast Colors for Dark Mode Compatibility

All diagrams MUST use dark backgrounds with white text (`color:#ffffff`) for maximum contrast and readability in both light and dark modes.

### Component Color Mapping

| Component Type | Fill Color | Stroke Color | Stroke Width | Text Color | Usage |
|---------------|------------|--------------|--------------|------------|--------|
| **Primary/Main** | `#e65100` | `#ff9800` | 3px | `#ffffff` | Main components, load balancers, orchestrators |
| **Active/Healthy** | `#1b5e20` | `#4caf50` | 2px | `#ffffff` | Active backends, healthy services |
| **Success State** | `#2e7d32` | `#66bb6a` | 2px | `#ffffff` | Successful operations, active components |
| **Failed/Unhealthy** | `#b71c1c` | `#f44336` | 2px | `#ffffff` | Failed components, error states |
| **Error/Alert** | `#d32f2f` | `#ef5350` | 2px | `#ffffff` | Errors, crashed services |
| **Data Storage** | `#0d47a1` | `#2196f3` | 2px | `#ffffff` | Redis, cache layers |
| **Database** | `#1a237e` | `#3f51b5` | 2px | `#ffffff` | PostgreSQL, persistent storage |
| **Client/External** | `#4a148c` | `#9c27b0` | 2px | `#ffffff` | CLI, TUI, API clients |
| **Special/Events** | `#880e4f` | `#c2185b` | 2px | `#ffffff` | WebSocket, special protocols |
| **Neutral/Info** | `#37474f` | `#78909c` | 2px | `#ffffff` | Stopped services, info boxes |
| **Warning** | `#ff6f00` | `#ffa726` | 2px | `#ffffff` | Decision points, warnings |

## Writing Style

### Core Principles

| Principle | Do | Don't |
|-----------|-----|-------|
| **Clarity** | "Click the Submit button" | "The submission interface should be utilized" |
| **Conciseness** | "Install dependencies: `npm install`" | "To install the required dependencies, run the npm install command" |
| **Active Voice** | "The API returns JSON data" | "JSON data is returned by the API" |
| **Specificity** | "Set timeout to 30 seconds" | "Set an appropriate timeout value" |
| **Consistency** | Use the same terms throughout | Mix "endpoint", "route", and "path" |

### Voice and Tone

- **Professional but approachable**: Technical accuracy without unnecessary jargon
- **Direct and actionable**: Focus on what the reader needs to do
- **Inclusive language**: Use "you" for instructions, avoid assumptions about expertise
- **Present tense**: Describe current behavior ("The system uses..." not "The system will use...")

### Callout Boxes

Use blockquotes with emoji indicators for different types of information:

> **📝 Note:** Additional context or information

> **⚠️ Warning:** Important caution about potential issues

> **✅ Tip:** Helpful suggestion or best practice

> **🚫 Deprecated:** Feature or method no longer recommended

> **🔒 Security:** Security-related information

## Code Examples

### Code Block Guidelines

Always specify the language for proper syntax highlighting:

````markdown
```rust
// Rust with type annotations
fn process_particle(particle: &Particle) -> Vec2 {
    particle.position + particle.velocity
}
```

```bash
#!/bin/bash
# Shell script with error handling
set -euo pipefail
cargo build --release || exit 1
```
````

## File Organization

### Standard Files

#### Root Level Documentation
- `README.md` - Project overview and entry point
- `CHANGELOG.md` - Version history and releases
- `CONTRIBUTING.md` - Contribution guidelines
- `LICENSE` - License information

#### Project-Specific
- `CLAUDE.md` - AI assistant instructions
- `docs/` - Extended documentation

## Maintenance

### Documentation Testing

#### Automated Checks
- Link validation (internal and external)
- Markdown linting (proper syntax)
- Code block syntax validation
- Mermaid diagram rendering

#### Manual Verification
- Technical accuracy review
- Example functionality testing
- Screenshot currency (if applicable)
- Flow diagram accuracy

## Review Checklist

### Pre-Commit Checklist

#### Structure & Format
- [ ] **Title**: Clear, descriptive H1 heading
- [ ] **Description**: Brief summary after title
- [ ] **TOC**: Present for documents > 500 words
- [ ] **Sections**: Logical hierarchy (H2 → H3 → H4)
- [ ] **Headings**: Descriptive and action-oriented

#### Content Quality
- [ ] **Clarity**: Simple, direct language
- [ ] **Completeness**: All necessary information included
- [ ] **Accuracy**: Technical details verified
- [ ] **Consistency**: Terminology matches project standards
- [ ] **Examples**: Practical and tested

#### Visual Elements
- [ ] **Diagrams**: Mermaid (not ASCII art)
- [ ] **Colors**: High contrast with white text
- [ ] **Tables**: Properly formatted with headers
- [ ] **Code blocks**: Language specified

#### Links & References
- [ ] **Internal links**: All working and accurate
- [ ] **External links**: Verified and accessible
- [ ] **Cross-references**: Updated in related docs
