# MCP Rust Analyzer Usage Examples

This document shows how to use the MCP rust-analyzer with Claude Code CLI.

## Starting the Server

Once installed, the MCP server will be available in Claude Code. You can interact with it using the `mcp:` prefix.

## Example Commands

### Code Analysis

```bash
# Get hover information for a position
mcp: get_hover examples/demo.rs:4:14

# Find all references to a symbol
mcp: find_references examples/demo.rs:11:9

# Analyze a symbol (searches across project)
mcp: analyze_symbol Person

# Get diagnostics for a file
mcp: get_diagnostics examples/demo.rs

# Find implementations of a trait
mcp: find_implementations examples/demo.rs:24:6
```

### Code Completion

```bash
# Get completions at a position
mcp: complete examples/demo.rs:7:20

# Get method signature help
mcp: signature_help examples/demo.rs:7:30

# Get context-based completions
mcp: get_completions "let x = Vec::"

# Resolve imports for a symbol
mcp: resolve_import HashMap

# Expand a code snippet
mcp: expand_snippet match_expr
```

### Refactoring

```bash
# Rename a symbol
mcp: rename Person Individual

# Extract a function
mcp: extract_function examples/demo.rs:6:5:9:10 calculate_sum

# Inline a variable or function
mcp: inline examples/demo.rs:7:9

# Organize imports
mcp: organize_imports examples/demo.rs
```

### Code Metrics and Analysis

```bash
# View project structure
mcp: project_structure

# Analyze dependencies
mcp: analyze_dependencies

# Get code metrics for a module
mcp: code_metrics src/

# Find dead code
mcp: find_dead_code

# Get improvement suggestions
mcp: suggest_improvements examples/demo.rs
```

## Advanced Usage

### Working with LSP

The MCP server uses rust-analyzer under the hood via the Language Server Protocol. When the `USE_LSP` environment variable is set to "true", it will spawn a rust-analyzer process and communicate with it.

### Debugging

To see detailed logs, set the log level:

```bash
RUST_LOG=debug mcp-rust-analyzer
```

## Integration with Claude Code

The server integrates seamlessly with Claude Code, providing:

1. **Context-aware suggestions**: Claude can see the structure of your Rust project
2. **Refactoring assistance**: Claude can help plan and execute refactorings
3. **Code understanding**: Claude can explain complex Rust concepts using project context
4. **Error resolution**: Claude can help fix compilation errors with full context

## Example Workflow

1. Open a Rust project in your terminal
2. Start Claude Code CLI
3. Ask Claude to analyze your code:
   ```
   "Can you explain what the Person struct does and suggest improvements?"
   ```
4. Claude will use the MCP server to:
   - Analyze the Person struct
   - Find all its usages
   - Check for potential improvements
   - Suggest refactorings

## Limitations

Currently, some features are still being implemented:
- Full refactoring support (extract function, inline)
- Live diagnostics streaming
- Workspace-wide symbol search
- Code actions integration

These will be added in future updates.