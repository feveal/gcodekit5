# GCodeKit5 - Detailed Remediation Plan

**Document Version**: 1.0  
**Date**: January 2026  
**Status**: Ready for Execution (No Dependencies Assumed Between Tasks)

---

## Overview

This document provides **85+ independently actionable tasks** to remediate all issues identified in CODEBASE_IMPROVEMENTS.md. Each task:
- ✅ Can be worked on independently
- ✅ Has minimal dependencies on other tasks
- ✅ Includes specific success criteria
- ✅ Has estimated effort in hours
- ✅ Is organized by category and priority

---

## Task Organization

- **P0 (Critical)**: Must complete in Q1 2026
- **P1 (High)**: Should complete in Q1-Q2 2026
- **P2 (Medium)**: Aim for Q2-Q3 2026
- **P3 (Low)**: Nice-to-have, can defer to 2027

---

# SECTION 1: ERROR HANDLING & ROBUSTNESS (CRITICAL)

## 1.1.1 - Audit and Document Unwrap Calls
**Category**: Error Handling | **Priority**: P0 | **Effort**: 4 hours

**Objective**: Create a comprehensive inventory of all 584 unwrap() calls with risk assessment.

**Task Steps**:
1. Run `grep -r "unwrap()" crates --include="*.rs" > /tmp/unwraps.txt`
2. Create spreadsheet: `unwrap_audit.csv` with columns:
   - File path
   - Line number
   - Function name
   - Context (UI/Network/File I/O/Core Logic)
   - Risk level (High/Medium/Low)
   - Suggested fix (Result/Option/Default)
3. Categorize by risk level
4. Identify top 100 highest-risk unwraps

**Success Criteria**:
- [ ] Complete inventory document created
- [ ] All 584 unwraps categorized
- [ ] Top 100 high-risk unwraps identified
- [ ] Risk categorization document committed to repo

**Dependencies**: None

**Testing**: Manual review

---

## 1.1.2 - Replace High-Risk Unwraps (Batch 1: UI/Network Layer)
**Category**: Error Handling | **Priority**: P0 | **Effort**: 12 hours

**Objective**: Replace unwraps in UI event handlers and network communication (highest risk areas).

**Target Files**:
- `crates/gcodekit5-ui/src/ui/gtk/designer.rs` (~10 unwraps)
- `crates/gcodekit5-communication/src/firmware/grbl/controller.rs` (~15 unwraps)
- `crates/gcodekit5-ui/src/ui/gtk/editor.rs` (~8 unwraps)

**Task Steps**:
1. For each file, identify unwrap pattern
2. Replace with appropriate error handling:
   - `.unwrap()` → `.unwrap_or_else(|e| { /* handle error */ })`
   - `.get().unwrap()` → `.get().ok_or_else(|| /* error */)?`
3. Add error logging via `tracing::error!`
4. Test each change

**Success Criteria**:
- [ ] 33 unwraps removed from UI/network layer
- [ ] Errors properly logged
- [ ] No functionality regression
- [ ] Tests pass

**Dependencies**: Task 1.1.1 (for priority list)

**Testing**:
- Unit tests for error paths
- Manual testing of error scenarios

---

## 1.1.3 - Replace Mid-Risk Unwraps (Batch 2: Data Processing)
**Category**: Error Handling | **Priority**: P0 | **Effort**: 10 hours

**Objective**: Replace unwraps in data processing layers (toolpath, geometry, visualization).

**Target Files**:
- `crates/gcodekit5-visualizer/src/gcode/mod.rs` (~15 unwraps)
- `crates/gcodekit5-designer/src/model.rs` (~12 unwraps)
- `crates/gcodekit5-designer/src/toolpath.rs` (~10 unwraps)

**Task Steps**:
1. Replace with `Result<T, E>` returns
2. Use `?` operator for error propagation
3. Add custom error types (see Task 1.2.1)
4. Document error cases in comments

**Success Criteria**:
- [ ] 37 unwraps removed from data layer
- [ ] Error cases documented
- [ ] Result types propagate correctly
- [ ] Tests pass

**Dependencies**: Task 1.2.1 (for error types) - Optional, can use anyhow::Result

**Testing**:
- Integration tests for error paths
- Edge case testing

---

## 1.1.4 - Replace Low-Risk Unwraps (Batch 3: Core Logic)
**Category**: Error Handling | **Priority**: P0 | **Effort**: 8 hours

**Objective**: Replace remaining unwraps in core business logic.

**Task Steps**:
1. Address unwraps in settings, device management, state
2. Use default values where appropriate
3. Add assertions for impossible cases with clear messages
4. Document expectations

**Success Criteria**:
- [ ] Remaining 200+ unwraps reviewed
- [ ] High-risk ones replaced (target 100 total)
- [ ] Assertions clarify code intent
- [ ] Documentation updated

**Dependencies**: None

**Testing**:
- Assertion testing in dev builds
- Manual verification

---

## 1.1.5 - Add CI Check for New Unwraps
**Category**: Error Handling | **Priority**: P0 | **Effort**: 2 hours

**Objective**: Prevent new unwraps from being introduced.

**Task Steps**:
1. Create `.github/workflows/clippy-check.yml`
2. Add job that:
   - Runs `cargo clippy -- -W clippy::unwrap_used`
   - Counts unwraps in diff
   - Fails if new unwraps detected
3. Update PR checklist to reference this check

**Success Criteria**:
- [ ] CI workflow created and tested
- [ ] Fails on new unwraps
- [ ] Can be overridden with `#[allow(unwrap_used)]` + justification
- [ ] PR template updated

**Dependencies**: None

**Testing**: Create test PR with unwrap to verify CI blocks it

---

## 1.2.1 - Define Error Types for Critical Crates
**Category**: Error Handling | **Priority**: P1 | **Effort**: 6 hours

**Objective**: Create proper error types using `thiserror` crate.

**Crates to Add Error Types**:
1. `gcodekit5-designer` - DesignError, GeometryError, ToolpathError
2. `gcodekit5-communication` - CommunicationError, ProtocolError
3. `gcodekit5-visualizer` - VisualizationError, ParsingError

**Task Steps**:
1. Create `src/error.rs` in each crate
2. Define error types:
```rust
#[derive(thiserror::Error, Debug)]
pub enum DesignError {
    #[error("Shape not found: {0}")]
    ShapeNotFound(String),
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("Toolpath generation failed: {0}")]
    ToolpathGeneration(#[from] anyhow::Error),
}
```
3. Export in `lib.rs`
4. Update public function returns to use new types

**Success Criteria**:
- [ ] Error types defined in 3 crates
- [ ] Implementations are complete
- [ ] Used in at least one function per type
- [ ] Tests verify error handling

**Dependencies**: None

**Testing**:
- Unit tests for error creation
- Integration tests for error propagation

---

## 1.2.2 - Add Context to Error Messages
**Category**: Error Handling | **Priority**: P1 | **Effort**: 4 hours

**Objective**: Use `anyhow::Context` for better error diagnostics.

**Task Steps**:
1. Identify I/O operations returning Result
2. Add `.context()` calls:
```rust
std::fs::read_to_string(path)
    .context("Failed to read design file")?
```
3. Add context in network operations
4. Document error context requirements

**Success Criteria**:
- [ ] All I/O operations have context
- [ ] Network ops have context
- [ ] Error messages are user-friendly
- [ ] Tests verify context appears in error output

**Dependencies**: None

**Testing**:
- Manual error scenario testing
- Verify error messages are informative

---

## 1.3.1 - Add State Validation Guards
**Category**: Error Handling | **Priority**: P1 | **Effort**: 5 hours

**Objective**: Add invariant checks to prevent invalid state transitions.

**Target Types**:
- `DesignerState` - Validate shape collections, selection
- `ControllerState` - Validate position, status consistency
- `CommunicatorState` - Validate connection state

**Task Steps**:
1. Add `assert_invariants()` method to each type
2. Call in setters and after mutations
3. Use `debug_assert!()` for performance-sensitive paths
4. Document invariants in comments

**Success Criteria**:
- [ ] Invariant methods added
- [ ] Called after state changes
- [ ] No false positives in tests
- [ ] Documentation explains expectations

**Dependencies**: None

**Testing**:
- Property-based tests
- State machine tests

---

# SECTION 2: CODE QUALITY & MAINTENANCE (HIGH PRIORITY)

## 2.1.1 - Fix Clippy Warning: impl Default [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 2 hours

**Objective**: Replace field assignment Default implementations with `derive(Default)`.

**Task Steps**:
1. Find all instances: `grep -n "impl Default for" crates --include="*.rs"`
2. For each, check if `derive(Default)` is applicable
3. Replace boilerplate with `#[derive(Default)]`
4. If not applicable, add comment explaining why

**Success Criteria**:
- [x] 8 Default impls converted to derive
- [x] Clippy warnings reduced (7 derivable_impls warnings eliminated)
- [x] Code size reduced
- [x] No behavior changes

**Dependencies**: None

**Testing**: Compile-only

**Completed**: 2026-01-19

---

## 2.1.2 - Fix Clippy Warning: Field Assignment Outside Initializer [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 3 hours

**Objective**: Restructure initializations to assign fields in initializer.

**Task Steps**:
1. Find instances where `Default::default()` used, then fields assigned
2. Rewrite as:
```rust
// Before
let mut obj = SomeStruct::default();
obj.field = value;

// After
let obj = SomeStruct {
    field: value,
    ..Default::default()
};
```
3. Verify no behavior changes

**Success Criteria**:
- [x] 18 instances refactored
- [x] Clippy warnings reduced (18 field_reassign_with_default warnings eliminated)
- [x] Code is more idiomatic
- [x] Tests pass

**Dependencies**: None

**Testing**: Compile + unit tests

**Completed**: 2026-01-19

---

## 2.1.3 - Fix Clippy Warning: Clamp Pattern [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Replace manual clamp logic with `.clamp()` method.

**Task Steps**:
1. Find instances: `grep -n "min\|max" crates --include="*.rs" | grep -E "if.*else"`
2. Replace:
```rust
// Before
let val = if x < min { min } else if x > max { max } else { x };

// After
let val = x.clamp(min, max);
```

**Success Criteria**:
- [x] 15 clamp patterns replaced
- [x] Code more readable
- [x] No behavior changes
- [x] Tests pass

**Dependencies**: None

**Testing**: Unit tests for boundary values

**Completed**: 2026-01-19

---

## 2.1.4 - Fix Clippy Warning: Derive Implementations [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 2 hours

**Objective**: Remove boilerplate impl blocks that can be derived.

**Task Steps**:
1. Find: `grep -B5 "impl.*Debug\|impl.*Clone\|impl.*Default" crates`
2. For each, check if `#[derive(...)]` is applicable
3. Replace impl with `#[derive(...)]`
4. Handle any conflicts with other derives

**Success Criteria**:
- [x] 5 boilerplate impls removed (Clone: 4, PartialEq+Hash: 2)
- [x] `#[derive(...)]` attributes added
- [x] Code shorter and clearer
- [x] Tests pass

**Dependencies**: None

**Testing**: Compile + unit tests

**Completed**: 2026-01-19

---

## 2.1.5 - Fix Clippy Warning: Copy Clones [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Remove unnecessary `.clone()` on Copy types.

**Task Steps**:
1. Find instances: `grep -n "\.clone()" crates --include="*.rs"`
2. For Copy types, replace with simple assignment
3. Verify all are truly Copy (test with `-Z print-type-sizes`)

**Success Criteria**:
- [x] 2 unnecessary clones removed
- [x] Code more efficient
- [x] No behavior changes
- [x] Tests pass

**Dependencies**: None

**Testing**: Compile

**Completed**: 2026-01-19

---

## 2.1.6 - Fix Clippy Warning: Identical If Blocks [✅ COMPLETED]
**Category**: Code Quality | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Simplify if expressions with identical branches.

**Task Steps**:
1. Find instances where branches do same thing
2. Simplify condition logic
3. Add comments if condition is important for clarity

**Success Criteria**:
- [x] All 5 collapsible blocks consolidated
- [x] Logic simplified
- [x] Tests still pass

**Dependencies**: None

**Testing**: Logic verification

**Completed**: 2026-01-19

---

## 2.2.1 - Modularize cam_tools.rs [✅ COMPLETED]
**Category**: Complexity | **Priority**: P0 | **Effort**: 16 hours

**Objective**: Split 5,837-line file into focused modules.

**Current Structure**:
```
cam_tools.rs (5,837 lines)
├─ Drill press tool (600 lines)
├─ Surface tool (800 lines)
├─ Boxing tool (700 lines)
└─ ...
```

**Target Structure**:
```
cam_tools/
├─ mod.rs (300 lines - exports)
├─ drill_press.rs (600 lines)
├─ surface.rs (800 lines)
├─ boxing.rs (700 lines)
├─ common.rs (200 lines - shared utilities)
└─ types.rs (100 lines - shared types)
```

**Task Steps**:
1. Create `crates/gcodekit5-ui/src/ui/gtk/cam_tools/` directory
2. Identify tool-specific sections in cam_tools.rs
3. Extract each into separate module
4. Create common.rs for shared code
5. Update mod.rs with re-exports
6. Update parent references

**Success Criteria**:
- [x] File split into 6+ modules (11 modules created)
- [x] Each module <1000 lines (largest is 965 lines)
- [x] No circular dependencies
- [x] Build passes successfully
- [x] Public API unchanged

**Completion Notes (January 2026)**:
Split into 11 modules totaling 6,038 lines:
- mod.rs (507 lines) - CamToolsView and exports
- common.rs (58 lines) - Shared utilities
- jigsaw.rs (555 lines) - JigsawTool
- bitmap_engraving.rs (828 lines) - BitmapEngravingTool
- vector_engraving.rs (965 lines) - VectorEngravingTool
- tabbed_box.rs (783 lines) - TabbedBoxMaker
- speeds_feeds.rs (196 lines) - SpeedsFeedsTool
- spoilboard_surfacing.rs (442 lines) - SpoilboardSurfacingTool
- spoilboard_grid.rs (419 lines) - SpoilboardGridTool
- gerber.rs (719 lines) - GerberTool
- drill_press.rs (566 lines) - DrillPressTool

**Dependencies**: None

**Testing**: Compile + full test suite

---

## 2.2.2 - Modularize designer.rs  
**Category**: Complexity | **Priority**: P0 | **Effort**: 20 hours

**Objective**: Refactor 5,791-line designer.rs, especially `new()` method (2000+ lines).

**Current Problem**:
- `DesignerView::new()` is 2000+ lines
- Mixed concerns: UI setup, event binding, layout configuration

**Target Structure**:
```
designer/
├─ mod.rs (builder pattern)
├─ canvas.rs (drawing area logic)
├─ toolbox.rs (tool selection - already partial)
├─ panels.rs (left/right panel setup)
├─ overlay.rs (floating controls)
├─ connections.rs (event handlers)
└─ builder.rs (construction phases)
```

**Task Steps**:
1. Create DesignerBuilder struct
2. Extract setup phases:
   - Phase 1: Create widgets
   - Phase 2: Configure layout
   - Phase 3: Connect signals
   - Phase 4: Setup overlays
3. Move each phase to builder method
4. Create builder module

**Success Criteria**:
- [ ] No single function >500 lines
- [ ] Logical separation of concerns
- [ ] Builder pattern clear
- [ ] All tests pass
- [ ] Performance unchanged

**Dependencies**: None

**Testing**: UI tests + visual verification

---

## 2.2.3 - Split designer_state.rs
**Category**: Complexity | **Priority**: P1 | **Effort**: 12 hours

**Objective**: Break 2,583-line state into manageable components.

**Current Problem**:
- Too many responsibilities
- Hard to find related code
- Difficult to test in isolation

**Target Structure**:
```
designer_state/
├─ mod.rs (exports, DesignerState)
├─ shapes.rs (shape management)
├─ selection.rs (selection logic)
├─ history.rs (undo/redo)
├─ viewport.rs (zoom/pan)
└─ tools.rs (tool state)
```

**Task Steps**:
1. Create module structure
2. Extract shape operations to shapes.rs
3. Extract selection logic to selection.rs
4. Move history logic to history.rs (coordinate with existing)
5. Move viewport to viewport.rs
6. Keep DesignerState as facade

**Success Criteria**:
- [ ] Each module <600 lines
- [ ] Clear module responsibilities
- [ ] Facade works correctly
- [ ] All tests pass
- [ ] No performance regression

**Dependencies**: None

**Testing**: Unit tests for each module

---

## 2.2.4 - Extract Property Editors
**Category**: Complexity | **Priority**: P1 | **Effort**: 10 hours

**Objective**: Split 2,671-line designer_properties.rs into focused editors.

**Target Structure**:
```
designer_properties/
├─ mod.rs (main panel)
├─ editors/
│  ├─ geometry.rs (geometry operations)
│  ├─ appearance.rs (fill, stroke)
│  ├─ dimensions.rs (size, position)
│  └─ effects.rs (fillet, offset, etc.)
└─ utils.rs (shared property widgets)
```

**Task Steps**:
1. Create editors submodule
2. Extract geometry editor
3. Extract appearance editor
4. Extract dimensions editor
5. Extract effects editor
6. Keep main panel as orchestrator

**Success Criteria**:
- [ ] Each editor <400 lines
- [ ] Clear purpose for each editor
- [ ] Main panel <300 lines
- [ ] Tests pass
- [ ] UI unchanged

**Dependencies**: None

**Testing**: UI tests

---

## 2.3.1 - Remove Debug Eprintln Calls
**Category**: Code Cleanup | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Remove all debug `eprintln!()` and `println!()` calls.

**Target Files**:
- `crates/gcodekit5-designer/src/stock_removal.rs` (8 calls)
- `crates/gcodekit5-ui/src/ui/gtk/editor.rs` (3 calls)
- Other scattered instances

**Task Steps**:
1. Find all: `grep -rn "eprintln!\|println!" crates --include="*.rs"`
2. For each:
   - If debug output, replace with `tracing::debug!`
   - If important info, use `tracing::info!`
   - If error context, use `tracing::error!`
3. Remove temporary debug markers

**Success Criteria**:
- [ ] All eprintln/println removed
- [ ] Replaced with tracing calls
- [ ] No DEBUG: prefixes in code
- [ ] Tests pass

**Dependencies**: None

**Testing**: Run code, verify no debug output to stderr

---

## 2.3.2 - Replace Debug Output with Structured Logging
**Category**: Code Cleanup | **Priority**: P1 | **Effort**: 2 hours

**Objective**: Convert debug prints to structured logging.

**Task Steps**:
1. Update imports to use tracing
2. Replace patterns:
```rust
// Before
eprintln!("DEBUG: Toolpath coordinate range: X:[{:.2}, {:.2}]", x_min, x_max);

// After
tracing::debug!(x_min, x_max, "toolpath coordinate range");
```
3. Verify structured fields appear in logs

**Success Criteria**:
- [ ] All debug output uses tracing
- [ ] Structured fields present
- [ ] Log output clean and readable
- [ ] Tests pass

**Dependencies**: Task 2.3.1

**Testing**: Run with RUST_LOG=debug

---

## 2.4.1 - Create GitHub Issues for TODOs
**Category**: Technical Debt | **Priority**: P0 | **Effort**: 3 hours

**Objective**: Convert all 30+ TODOs to tracked GitHub issues.

**Task Steps**:
1. Extract all TODOs:
```bash
grep -rn "TODO\|FIXME" crates --include="*.rs" > /tmp/todos.txt
```
2. For each TODO:
   - Create GitHub issue with title and description
   - Include context from code
   - Estimate effort
   - Add label (e.g., `type:enhancement`, `area:designer`)
3. Update code comments to link to issue:
```rust
// TODO: Fix xyz - see issue #123
```

**Success Criteria**:
- [ ] 30+ issues created
- [ ] All TODOs linked to issues
- [ ] Issues have clear description
- [ ] Effort estimates added

**Dependencies**: None

**Testing**: Manual verification

---

## 2.4.2 - Implement Critical TODOs - Listener Registration
**Category**: Technical Debt | **Priority**: P0 | **Effort**: 6 hours

**Location**: `crates/gcodekit5-communication/src/firmware/grbl/controller.rs`

**Objective**: Implement listener registration/unregistration (currently stub).

**Task Steps**:
1. Review existing listener architecture
2. Implement `register_listener()` method
3. Implement `unregister_listener()` method
4. Add tests for listener management
5. Ensure listeners receive events

**Success Criteria**:
- [ ] Methods fully implemented
- [ ] Unit tests for registration/unregistration
- [ ] Integration tests with real events
- [ ] No memory leaks (listeners cleanup)

**Dependencies**: None

**Testing**:
- Unit tests for listener management
- Integration tests with controller

---

## 2.4.3 - Implement Critical TODOs - Toolpath Generation Integration
**Category**: Technical Debt | **Priority**: P1 | **Effort**: 8 hours

**Location**: `crates/gcodekit5-designer/src/slice_toolpath.rs` (3 TODOs)

**Objective**: Integrate toolpath generation with ToolpathGenerator.

**Task Steps**:
1. Review ToolpathGenerator interface
2. Integrate slice_toolpath module
3. Update tests to use proper generator
4. Remove TODO comments

**Success Criteria**:
- [ ] Integration complete
- [ ] 3 TODOs removed
- [ ] Tests pass
- [ ] No functionality loss

**Dependencies**: None

**Testing**: Integration tests

---

## 2.4.4 - Implement Critical TODOs - File Operations
**Category**: Technical Debt | **Priority**: P1 | **Effort**: 5 hours

**Location**: `crates/gcodekit5-designer/src/designer_state.rs`

**Objective**: Implement file I/O operations (currently stubbed).

**Task Steps**:
1. Implement `save()` method
2. Implement `load()` method
3. Add proper error handling
4. Test round-trip save/load

**Success Criteria**:
- [ ] Methods fully implemented
- [ ] Round-trip testing passes
- [ ] Error handling in place
- [ ] File format documented

**Dependencies**: Task 1.2.1 (error types)

**Testing**: Integration tests for file operations

---

# SECTION 3: TYPE SYSTEM & API DESIGN

## 3.1.1 - Create Type Aliases for Complex Types
**Category**: Type Design | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Reduce readability burden of complex nested types.

**Task Steps**:
1. Identify top 20 complex types:
   - `Rc<RefCell<Box<dyn Component>>>`
   - `Arc<Mutex<State>>`
   - etc.
2. Create module `gcodekit5-core/src/types/aliases.rs`
3. Define aliases:
```rust
pub type ComponentRef = Rc<RefCell<Box<dyn Component>>>;
pub type StateRef = Arc<Mutex<AppState>>;
```
4. Update imports in crates
5. Document rationale

**Success Criteria**:
- [ ] 20+ aliases created
- [ ] Used consistently
- [ ] Code more readable
- [ ] Tests pass

**Dependencies**: None

**Testing**: Compile

---

## 3.1.2 - Replace Box<dyn T> Where Concrete Type Possible
**Category**: Type Design | **Priority**: P2 | **Effort**: 6 hours

**Objective**: Remove unnecessary trait objects where concrete types work.

**Task Steps**:
1. Audit Box<dyn T> usages
2. For each:
   - Check if only one implementation
   - If yes, use concrete type instead
   - If multiple, keep but document why
3. Update call sites

**Success Criteria**:
- [ ] 5+ trait objects removed
- [ ] Performance improved
- [ ] Type safety improved
- [ ] Tests pass

**Dependencies**: None

**Testing**: Compile + unit tests

---

## 3.2.1 - Document Core Crate Public APIs
**Category**: Documentation | **Priority**: P1 | **Effort**: 8 hours

**Objective**: Add `///` documentation to 165+ public APIs in core crate.

**Target Coverage**: 100% of public items

**Task Steps**:
1. Extract list of public items:
```bash
grep -n "pub fn\|pub struct\|pub enum\|pub trait" crates/gcodekit5-core/src --include="*.rs" > /tmp/apis.txt
```
2. For each, add doc comment:
```rust
/// Represents a CNC position in work coordinates.
///
/// # Fields
/// - `x`, `y`, `z`: Linear axes in millimeters
///
/// # Example
/// ```
/// let pos = Position::new(10.0, 20.0, 0.0, 0.0, 0.0, 0.0);
/// ```
pub struct Position {
    // ...
}
```
3. Run `cargo doc --open` to verify

**Success Criteria**:
- [ ] All 165+ public items documented
- [ ] Examples for major types
- [ ] Error cases explained
- [ ] `cargo doc` builds successfully

**Dependencies**: None

**Testing**: `cargo test --doc` for doc examples

---

## 3.2.2 - Add Module-Level Documentation
**Category**: Documentation | **Priority**: P2 | **Effort**: 4 hours

**Objective**: Add `//!` docs to each module explaining purpose.

**Task Steps**:
1. For each module file, add header:
```rust
//! # Designer State
//!
//! This module manages the state of the visual designer, including:
//! - Active shapes and selection
//! - Undo/redo history
//! - Canvas viewport (zoom, pan)
//!
//! ## Thread Safety
//! Not Send/Sync - UI thread only.
```
2. Document key types and functions

**Success Criteria**:
- [ ] All modules documented
- [ ] Clear purpose statements
- [ ] Thread-safety noted where relevant
- [ ] `cargo doc` looks good

**Dependencies**: None

**Testing**: `cargo doc`

---

## 3.3.1 - Create Builder for ControllerStatus
**Category**: Type Design | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Replace Default + field assignment with builder pattern.

**Current Usage**:
```rust
let mut status = ControllerStatus::default();
status.position.x = 10.0;
status.feed_rate = 100;
// 12+ fields...
```

**Target Usage**:
```rust
let status = ControllerStatusBuilder::new()
    .position_x(10.0)
    .feed_rate(100)
    .build();
```

**Task Steps**:
1. Create builder struct
2. Add builder methods
3. Implement `build()` validation
4. Update call sites
5. Document builder

**Success Criteria**:
- [ ] Builder implemented
- [ ] Type-safe configuration
- [ ] Validation in build()
- [ ] Tests pass

**Dependencies**: None

**Testing**: Unit tests for builder

---

## 3.3.2 - Create Builder for ToolpathSettings
**Category**: Type Design | **Priority**: P2 | **Effort**: 2 hours

**Objective**: Create builder for 8+ parameter type.

**Task Steps**:
1. Create ToolpathSettingsBuilder
2. Add methods for each field
3. Implement validation in build()
4. Add documentation

**Success Criteria**:
- [ ] Builder implemented
- [ ] Clear, chainable API
- [ ] Validation works
- [ ] Tests pass

**Dependencies**: None

**Testing**: Unit tests

---

# SECTION 4: TESTING & COVERAGE

## 4.1.1 - Establish Testing Framework & Metrics
**Category**: Testing | **Priority**: P0 | **Effort**: 4 hours

**Objective**: Set up infrastructure to measure and enforce test coverage.

**Task Steps**:
1. Add `cargo-tarpaulin` to dev dependencies
2. Create CI workflow `.github/workflows/coverage.yml`:
   - Run `cargo tarpaulin`
   - Generate coverage report
   - Comment on PRs with results
3. Set coverage targets per crate:
   - Core: 80%
   - Designer: 70%
   - Communication: 75%
4. Create coverage baseline

**Success Criteria**:
- [ ] Coverage measurement working
- [ ] CI reports coverage
- [ ] Baseline established
- [ ] Coverage report generation working

**Dependencies**: None

**Testing**: Create test PR to verify CI

---

## 4.1.2 - Document Testing Strategy
**Category**: Testing | **Priority**: P0 | **Effort**: 3 hours

**Objective**: Create testing strategy document with patterns and examples.

**Document Contents**:
1. Testing philosophy
2. Test organization (unit/integration/doc)
3. Naming conventions
4. Mocking patterns
5. Property-based testing guide
6. Performance test approach

**Task Steps**:
1. Create `TESTING.md` in repo
2. Include examples for each pattern
3. Reference in CONTRIBUTING.md
4. Get team feedback

**Success Criteria**:
- [ ] Document created
- [ ] Examples provided
- [ ] CI references document
- [ ] Team reviews and approves

**Dependencies**: None

**Testing**: Manual review

---

## 4.1.3 - Setup Property-Based Testing
**Category**: Testing | **Priority**: P1 | **Effort**: 3 hours

**Objective**: Add `proptest` dependency and create generator patterns.

**Task Steps**:
1. Add `proptest` to dev-dependencies
2. Create `crates/gcodekit5-core/src/lib.rs` test utilities:
```rust
#[cfg(test)]
mod proptest_strategies {
    use proptest::prelude::*;
    
    pub fn arb_position() -> impl Strategy<Value = Position> {
        // Generate valid positions
    }
}
```
3. Create 3 property-based tests as examples
4. Document usage

**Success Criteria**:
- [ ] proptest integrated
- [ ] Generators created
- [ ] Example tests written
- [ ] Tests pass

**Dependencies**: None

**Testing**: Run proptest suite

---

## 4.2.1 - Add Designer State Integration Tests
**Category**: Testing | **Priority**: P1 | **Effort**: 8 hours

**Objective**: Test designer operations end-to-end (copy, paste, delete, group).

**Test Coverage**:
- [ ] Create shape workflow
- [ ] Copy/paste shape
- [ ] Delete shape
- [ ] Group/ungroup shapes
- [ ] Undo/redo sequence
- [ ] Selection consistency

**Task Steps**:
1. Create `crates/gcodekit5-designer/tests/designer_operations.rs`
2. Write tests for each operation
3. Test state consistency before/after
4. Test error conditions

**Success Criteria**:
- [ ] 10+ integration tests
- [ ] All operations covered
- [ ] State consistency verified
- [ ] 90% operation coverage

**Dependencies**: None

**Testing**: Run test suite

---

## 4.2.2 - Add Toolpath Generation Tests
**Category**: Testing | **Priority**: P1 | **Effort**: 6 hours

**Objective**: Test toolpath generation with various shapes.

**Scenarios**:
- [ ] Simple rectangle
- [ ] Circle with lead-in
- [ ] Complex path
- [ ] Multiple shapes
- [ ] Error conditions

**Task Steps**:
1. Create `crates/gcodekit5-designer/tests/toolpath_generation.rs`
2. Create test fixtures
3. Write parametrized tests
4. Verify output consistency

**Success Criteria**:
- [ ] 15+ test scenarios
- [ ] Error paths tested
- [ ] Output verified
- [ ] Deterministic results

**Dependencies**: None

**Testing**: Run tests

---

## 4.2.3 - Add Error Recovery Tests
**Category**: Testing | **Priority**: P1 | **Effort**: 5 hours

**Objective**: Test error handling in key operations.

**Scenarios**:
- [ ] Network interruption during file send
- [ ] File not found during load
- [ ] Malformed G-code during parse
- [ ] Geometry errors
- [ ] Resource exhaustion

**Task Steps**:
1. Create `crates/*/tests/error_scenarios.rs`
2. Mock error conditions
3. Verify recovery behavior
4. Document error expectations

**Success Criteria**:
- [ ] 20+ error scenarios
- [ ] Recovery verified
- [ ] Error messages helpful
- [ ] No silent failures

**Dependencies**: None

**Testing**: Error injection tests

---

## 4.3.1 - Setup Mutation Testing
**Category**: Testing | **Priority**: P2 | **Effort**: 2 hours

**Objective**: Add `cargo-mutants` for test effectiveness analysis.

**Task Steps**:
1. Install `cargo install cargo-mutants`
2. Create CI job (optional, runs manually)
3. Run on critical crates
4. Document results
5. Identify weak tests

**Success Criteria**:
- [ ] Mutation testing configured
- [ ] Baseline run complete
- [ ] Results documented
- [ ] Weak tests identified

**Dependencies**: None

**Testing**: Run cargo-mutants

---

## 4.3.2 - Improve Tests Identified by Mutation
**Category**: Testing | **Priority**: P2 | **Effort**: 4 hours

**Objective**: Enhance tests based on mutation analysis results.

**Task Steps**:
1. Analyze mutation report
2. Identify weak assertions
3. Add boundary tests
4. Test all code paths
5. Re-run mutations

**Success Criteria**:
- [ ] Mutation kill rate >80%
- [ ] All code paths tested
- [ ] Boundary conditions covered
- [ ] Re-run shows improvement

**Dependencies**: Task 4.3.1

**Testing**: Mutation analysis

---

# SECTION 5: PERFORMANCE & OPTIMIZATION

## 5.1.1 - Profile Designer Toolpath Generation
**Category**: Performance | **Priority**: P1 | **Effort**: 4 hours

**Objective**: Identify hotspots in toolpath generation.

**Task Steps**:
1. Create benchmark shapes (100 points, 1000 points, etc.)
2. Run profiler:
```bash
cargo build --release
flamegraph --bin gcodekit5 --freq 97 -- -profile-toolpath
```
3. Identify hotspots
4. Document findings

**Success Criteria**:
- [ ] Flamegraph generated
- [ ] Hotspots identified
- [ ] Report documented
- [ ] Shared with team

**Dependencies**: None

**Testing**: Manual profiling

---

## 5.1.2 - Optimize Identified Hotspot (Toolpath)
**Category**: Performance | **Priority**: P1 | **Effort**: 8 hours

**Objective**: Improve performance of top identified hotspot.

**Potential Optimizations**:
- Cache geometry calculations
- Use SIMD for vector operations
- Reduce allocations
- Batch operations

**Task Steps**:
1. Identify specific bottleneck
2. Implement optimization
3. Benchmark before/after
4. Verify correctness
5. Document change

**Success Criteria**:
- [ ] 20%+ performance improvement
- [ ] Correctness verified
- [ ] Benchmarks show improvement
- [ ] No memory regression

**Dependencies**: Task 5.1.1

**Testing**:
- Performance benchmarks
- Correctness tests

---

## 5.1.3 - Profile Visualizer Rendering
**Category**: Performance | **Priority**: P1 | **Effort**: 3 hours

**Objective**: Identify rendering hotspots.

**Task Steps**:
1. Load large G-code file (10K+ lines)
2. Profile rendering
3. Identify bottlenecks
4. Document findings

**Success Criteria**:
- [ ] Hotspots identified
- [ ] Report created
- [ ] Optimization opportunities listed
- [ ] Shared with team

**Dependencies**: None

**Testing**: Manual profiling

---

## 5.1.4 - Optimize Designer Hit Testing
**Category**: Performance | **Priority**: P1 | **Effort**: 5 hours

**Objective**: Improve shape selection speed with 1000+ shapes.

**Current Issue**: Linear search through all shapes

**Task Steps**:
1. Implement spatial index (quadtree or BVH)
2. Cache hit test results
3. Invalidate cache on shape changes
4. Benchmark

**Success Criteria**:
- [ ] Hit testing 10x faster
- [ ] Memory usage reasonable
- [ ] No visual differences
- [ ] Tests pass

**Dependencies**: None

**Testing**:
- Performance tests
- Correctness verification

---

## 5.2.1 - Audit String Allocations
**Category**: Performance | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Identify opportunities to reduce string allocations.

**Task Steps**:
1. Use `cargo flamegraph` with allocations
2. Find string allocation hotspots
3. Identify candidates for string interning
4. Document findings

**Success Criteria**:
- [ ] Hotspots identified
- [ ] Report created
- [ ] Optimization plan made
- [ ] Recommendations documented

**Dependencies**: None

**Testing**: Profiling

---

## 5.2.2 - Intern Frequently-Used Strings
**Category**: Performance | **Priority**: P2 | **Effort**: 4 hours

**Objective**: Reduce string allocation overhead via interning.

**Task Steps**:
1. Identify high-frequency strings (e.g., "Error", "Designer")
2. Create string pool/intern
3. Use interned strings
4. Benchmark memory usage

**Success Criteria**:
- [ ] Interning implemented
- [ ] Memory usage reduced
- [ ] Performance stable/improved
- [ ] Tests pass

**Dependencies**: None

**Testing**: Memory profiling

---

## 5.2.3 - Reduce Geometry Vector Allocations
**Category**: Performance | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Use `SmallVec` for vectors that rarely exceed 16 elements.

**Task Steps**:
1. Find vector types storing points, vertices
2. Replace `Vec<T>` with `SmallVec<[T; 16]>`
3. Benchmark memory allocation
4. Verify performance

**Success Criteria**:
- [ ] SmallVec integrated
- [ ] Allocations reduced
- [ ] Performance improved
- [ ] Tests pass

**Dependencies**: None

**Testing**: Memory profiling

---

## 5.3.1 - Setup Criterion Benchmarks
**Category**: Performance | **Priority**: P2 | **Effort**: 4 hours

**Objective**: Create performance benchmarks to track regressions.

**Benchmarks to Create**:
1. Toolpath generation (100, 1000 points)
2. G-code parsing (small, large files)
3. Geometry operations (union, difference, etc.)
4. State updates (add 100 shapes)

**Task Steps**:
1. Add `criterion` to dev-dependencies
2. Create `benches/` directory
3. Write benchmark code
4. Run baseline
5. Document setup

**Success Criteria**:
- [ ] 5+ benchmarks created
- [ ] Baseline established
- [ ] Setup documented
- [ ] Runs in CI (optional)

**Dependencies**: None

**Testing**: Run benchmarks

---

## 5.3.2 - Add CI Benchmark Tracking (Optional)
**Category**: Performance | **Priority**: P3 | **Effort**: 3 hours

**Objective**: Track benchmarks across commits (optional).

**Task Steps**:
1. Setup benchmark CI job
2. Store baseline
3. Compare new runs to baseline
4. Comment on PRs if >5% regression

**Success Criteria**:
- [ ] CI job created
- [ ] Benchmarks run on each commit
- [ ] Comparisons reported
- [ ] Alerts on regressions

**Dependencies**: Task 5.3.1

**Testing**: Create test PR

---

# SECTION 6: ARCHITECTURE & DESIGN PATTERNS

## 6.1.1 - Design Event Bus System
**Category**: Architecture | **Priority**: P1 | **Effort**: 6 hours

**Objective**: Design decoupled event system (not implementation, just design).

**Deliverable**: Architecture document with:
1. Event types
2. Subscriber interface
3. Event bus API
4. Usage patterns
5. Diagram
6. Comparison to current approach

**Task Steps**:
1. Review current callback chains
2. Identify event categories
3. Design event types
4. Create Subscriber trait
5. Design EventBus struct
6. Document patterns
7. Create diagrams

**Success Criteria**:
- [ ] Design document created
- [ ] Event types identified
- [ ] API designed
- [ ] Team reviews and approves
- [ ] Ready for implementation

**Dependencies**: None

**Testing**: Design review

---

## 6.1.2 - Implement Core Event Bus
**Category**: Architecture | **Priority**: P1 | **Effort**: 8 hours

**Objective**: Implement the designed event bus.

**Task Steps**:
1. Create `gcodekit5-core/src/event_bus.rs`
2. Implement EventBus struct
3. Implement Subscriber trait
4. Add event dispatch logic
5. Create unit tests
6. Document usage

**Success Criteria**:
- [ ] Core bus implemented
- [ ] Tests pass
- [ ] Documentation clear
- [ ] Ready for integration

**Dependencies**: Task 6.1.1

**Testing**: Unit tests

---

## 6.1.3 - Integrate Event Bus into Designer
**Category**: Architecture | **Priority**: P2 | **Effort**: 12 hours

**Objective**: Replace callback chains with event bus.

**Events to Implement**:
- ShapeCreated
- ShapeDeleted
- SelectionChanged
- GcodeGenerated
- StateChanged

**Task Steps**:
1. Create Designer event types
2. Create DesignerSubscriber
3. Replace callbacks with events
4. Add event dispatch in operations
5. Test integration
6. Remove old callbacks

**Success Criteria**:
- [ ] Callbacks replaced
- [ ] Events working
- [ ] Tests pass
- [ ] No functionality loss

**Dependencies**: Task 6.1.2

**Testing**: Integration tests

---

## 6.2.1 - Separate Designer Business Logic
**Category**: Architecture | **Priority**: P1 | **Effort**: 10 hours

**Objective**: Extract pure business logic from UI layer.

**Create Module**: `crates/gcodekit5-designer/src/business_logic/`

**Functions to Extract**:
- copy_shapes(state, shape_ids) → Vec<Shape>
- paste_shapes(state, shapes) → Result<Vec<ShapeId>>
- delete_shapes(state, shape_ids) → Result<()>
- group_shapes(state, shape_ids) → Result<GroupId>
- apply_geometry_op(shape, op) → Result<Shape>

**Task Steps**:
1. Create business_logic module
2. Extract functions from UI handlers
3. Make pure (no UI dependencies)
4. Add comprehensive tests
5. Update UI to call business logic

**Success Criteria**:
- [ ] 10+ functions extracted
- [ ] Pure functions (no side effects)
- [ ] Well-tested
- [ ] UI updated
- [ ] Easier to test

**Dependencies**: None

**Testing**: Unit tests for business logic

---

## 6.2.2 - Separate Visualizer Business Logic
**Category**: Architecture | **Priority**: P1 | **Effort**: 8 hours

**Objective**: Extract visualization concerns from rendering.

**Create Module**: `crates/gcodekit5-visualizer/src/business_logic/`

**Functions to Extract**:
- compute_bounds(gcode) → Bounds
- generate_tool_path_points(gcode) → Vec<Point3D>
- apply_stock_removal(gcode, stock) → Gcode
- compute_feed_rate_stats(gcode) → Stats

**Task Steps**:
1. Create module
2. Extract functions
3. Remove rendering dependencies
4. Add tests
5. Update rendering layer

**Success Criteria**:
- [ ] Functions extracted
- [ ] Pure logic
- [ ] Tests added
- [ ] Rendering updated
- [ ] Performance maintained

**Dependencies**: None

**Testing**: Unit tests

---

## 6.2.3 - Create UI Presentation Layer Documentation
**Category**: Architecture | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Document pattern for UI layer (thin facade over business logic).

**Document Contents**:
1. Responsibilities of UI layer
2. How to call business logic
3. Event handling patterns
4. Error display patterns
5. Code examples

**Task Steps**:
1. Create `ARCHITECTURE_UI_LAYER.md`
2. Include code examples
3. Document anti-patterns
4. Get team review

**Success Criteria**:
- [ ] Document created
- [ ] Examples clear
- [ ] Team approves
- [ ] Referenced in guidelines

**Dependencies**: None

**Testing**: Manual review

---

# SECTION 7: DEPENDENCY MANAGEMENT

## 7.1.1 - Audit Dependencies
**Category**: Dependencies | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Comprehensive audit of all dependencies.

**Task Steps**:
1. Run `cargo tree` to see dependency tree
2. Run `cargo udeps` to find unused deps
3. Create `dependency_audit.md` with:
   - Duplicate versions
   - Unused dependencies
   - Large dependencies
   - Security audit results
4. Make recommendations

**Success Criteria**:
- [ ] Audit complete
- [ ] Report created
- [ ] Unused deps identified
- [ ] Duplicates found
- [ ] Recommendations made

**Dependencies**: None

**Testing**: Manual review

---

## 7.1.2 - Remove Unused Dependencies
**Category**: Dependencies | **Priority**: P2 | **Effort**: 2 hours

**Objective**: Remove identified unused dependencies.

**Task Steps**:
1. For each unused dependency from audit
2. Remove from Cargo.toml
3. Run tests to verify no impact
4. Commit

**Success Criteria**:
- [ ] Unused deps removed
- [ ] Tests still pass
- [ ] Build time possibly reduced
- [ ] Dependency report cleaner

**Dependencies**: Task 7.1.1

**Testing**: Full test suite

---

## 7.1.3 - Consolidate Duplicate Dependency Versions
**Category**: Dependencies | **Priority**: P2 | **Effort**: 1 hour

**Objective**: Update duplicates to single version.

**Task Steps**:
1. From audit, find duplicate versions
2. Update to single version (prefer latest compatible)
3. Run tests
4. Verify no issues

**Success Criteria**:
- [ ] Duplicates consolidated
- [ ] Tests pass
- [ ] Smaller Cargo.lock
- [ ] Build potentially faster

**Dependencies**: Task 7.1.1

**Testing**: Full test suite

---

## 7.2.1 - Setup Dependabot
**Category**: Dependencies | **Priority**: P2 | **Effort**: 1 hour

**Objective**: Auto-update dependencies.

**Task Steps**:
1. Create `.github/dependabot.yml`:
```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
```
2. Configure to create PRs
3. Setup auto-merge for minor/patch

**Success Criteria**:
- [ ] Dependabot configured
- [ ] Creates PRs automatically
- [ ] CI runs on updates
- [ ] Team receives notifications

**Dependencies**: None

**Testing**: Verify Dependabot creates test PR

---

## 7.2.2 - Manual Monthly Dependency Review
**Category**: Dependencies | **Priority**: P2 | **Effort**: 2 hours (monthly)

**Objective**: Regular dependency maintenance.

**Task Steps**:
1. Run `cargo outdated`
2. Review available updates
3. Check for security advisories
4. Update major versions carefully
5. Test thoroughly
6. Document changes

**Success Criteria**:
- [ ] Review completed
- [ ] Security updates applied
- [ ] Tests pass
- [ ] CHANGELOG updated

**Dependencies**: None

**Testing**: Full test suite

---

## 7.3.1 - Set MSRV (Minimum Supported Rust Version)
**Category**: Dependencies | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Define and document minimum Rust version.

**Task Steps**:
1. Determine lowest supported version (current: 1.70)
2. Add to `Cargo.toml`:
```toml
[package]
rust-version = "1.70"
```
3. Create CI job to test on MSRV version
4. Document in README

**Success Criteria**:
- [ ] MSRV set in Cargo.toml
- [ ] CI tests MSRV
- [ ] README documents MSRV
- [ ] Verified working on MSRV

**Dependencies**: None

**Testing**: Build on MSRV version

---

# SECTION 8: DOCUMENTATION & DX

## 8.1.1 - Create Architecture Decision Records (ADRs)
**Category**: Documentation | **Priority**: P2 | **Effort**: 6 hours

**ADRs to Create**:
1. ADR-001: GTK4 vs other UI frameworks
2. ADR-002: Coordinate system (Y-flip)
3. ADR-003: Modular crates structure
4. ADR-004: Interior mutability patterns
5. ADR-005: Error handling strategy

**Task Steps**:
1. Create `docs/adr/` directory
2. Write each ADR (template provided)
3. Get team review
4. Link from README

**Success Criteria**:
- [ ] 5+ ADRs created
- [ ] Team reviews
- [ ] Linked from README
- [ ] Format consistent

**Dependencies**: None

**Testing**: Manual review

---

## 8.1.2 - Document Coordinate System
**Category**: Documentation | **Priority**: P2 | **Effort**: 2 hours

**Objective**: Explain Designer vs GTK coordinate systems clearly.

**Task Steps**:
1. Create detailed explanation with diagrams
2. Document Y-flip logic
3. Show examples of conversions
4. Explain implications for rotations
5. Add to ADR or GTK4.md

**Success Criteria**:
- [ ] Explanation clear
- [ ] Diagrams created
- [ ] Examples provided
- [ ] Team understands

**Dependencies**: None

**Testing**: Manual review

---

## 8.2.1 - Create Developer Setup Guide
**Category**: Documentation | **Priority**: P2 | **Effort**: 3 hours

**Objective**: Make it easy for new developers to get started.

**Create**: `DEVELOPMENT.md` with:
1. Prerequisites
2. Clone and build instructions
3. Running tests
4. Running the application
5. Debugging tips
6. Common issues

**Task Steps**:
1. Document current setup
2. Test instructions on clean system
3. Verify completeness
4. Get feedback from new dev if possible

**Success Criteria**:
- [ ] Guide created
- [ ] Instructions tested
- [ ] Screenshots if helpful
- [ ] Troubleshooting section

**Dependencies**: None

**Testing**: Manual - follow guide

---

## 8.2.2 - Create Contributing Guidelines
**Category**: Documentation | **Priority**: P2 | **Effort**: 2 hours

**Create**: `CONTRIBUTING.md` with:
1. Code style requirements
2. Branch naming
3. PR process
4. Commit message format
5. Testing requirements
6. Documentation expectations

**Task Steps**:
1. Consolidate existing guidelines
2. Add missing pieces
3. Examples for each
4. Get team consensus

**Success Criteria**:
- [ ] Guide created
- [ ] Clear expectations
- [ ] Examples provided
- [ ] Team approves

**Dependencies**: None

**Testing**: Manual review

---

## 8.2.3 - Create Architecture Overview
**Category**: Documentation | **Priority**: P2 | **Effort**: 4 hours

**Create**: `ARCHITECTURE.md` with:
1. Crate dependencies
2. Data flow diagrams
3. Key design patterns
4. Threading model
5. Error handling strategy
6. References to ADRs

**Task Steps**:
1. Draw crate dependency graph
2. Document main flows
3. Explain key patterns
4. Create diagrams (ASCII or images)
5. Explain threading model

**Success Criteria**:
- [ ] Document created
- [ ] Diagrams clear
- [ ] Comprehensive
- [ ] Team reviews

**Dependencies**: None

**Testing**: Manual review

---

## 8.3.1 - Create User Guide Structure
**Category**: Documentation | **Priority**: P3 | **Effort**: 4 hours

**Objective**: Plan and structure user documentation.

**Task Steps**:
1. Outline user guide chapters
2. Create `docs/user/` directory
3. Write placeholders for each chapter
4. Get feedback on structure

**Success Criteria**:
- [ ] Structure created
- [ ] Chapters identified
- [ ] Team approves
- [ ] Ready for content writing

**Dependencies**: None

**Testing**: Manual review

---

# SECTION 9: TOOLING & WORKFLOW

## 9.1.1 - Create Pre-commit Hook
**Category**: Tooling | **Priority**: P0 | **Effort**: 2 hours

**Objective**: Prevent commits with common issues.

**Create**: `.githooks/pre-commit` that runs:
1. `cargo fmt --check`
2. `cargo clippy --all` (warnings don't block)
3. `cargo test --lib` (unit tests only, fast)

**Task Steps**:
1. Create hook script
2. Update git config to use hooks: `git config core.hooksPath .githooks`
3. Make executable
4. Document in README/CONTRIBUTING
5. Add optional skip: `git commit --no-verify`

**Success Criteria**:
- [ ] Hook works locally
- [ ] Prevents bad commits
- [ ] Performance acceptable
- [ ] Team uses it

**Dependencies**: None

**Testing**: Manual testing

---

## 9.1.2 - Setup GitHub PR Template
**Category**: Tooling | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Guide PR authors and reviewers.

**Create**: `.github/pull_request_template.md` with:
1. Description checklist
2. PR checklist (copy from CODEBASE_IMPROVEMENTS.md)
3. Testing instructions
4. Related issues
5. Screenshots (if UI change)

**Task Steps**:
1. Create template
2. Add helpful sections
3. Commit to repo

**Success Criteria**:
- [ ] Template created
- [ ] Used by PRs
- [ ] Improves PR quality

**Dependencies**: None

**Testing**: Create test PR

---

## 9.2.1 - Setup Issue Templates
**Category**: Tooling | **Priority**: P1 | **Effort**: 1 hour

**Objective**: Standardize issue reporting.

**Create** `.github/ISSUE_TEMPLATE/`:
1. `bug_report.md`
2. `feature_request.md`
3. `documentation.md`

**Task Steps**:
1. Create templates
2. Add helpful fields
3. Include examples

**Success Criteria**:
- [ ] Templates created
- [ ] Users select template
- [ ] Issues more complete

**Dependencies**: None

**Testing**: Create test issues

---

## 9.3.1 - Add Development Container
**Category**: Tooling | **Priority**: P3 | **Effort**: 2 hours

**Objective**: One-click development environment.

**Create**: `.devcontainer/` with:
1. `Dockerfile` (Rust + GTK4 dev environment)
2. `devcontainer.json` (VS Code config)

**Task Steps**:
1. Create Dockerfile
2. Create devcontainer.json
3. Document in README
4. Test in VS Code

**Success Criteria**:
- [ ] Container builds
- [ ] Development works in container
- [ ] VS Code opens automatically
- [ ] Faster onboarding

**Dependencies**: None

**Testing**: Manual in VS Code

---

---

# SUMMARY OF ALL TASKS

## Task Count by Category

| Category | P0 | P1 | P2 | P3 | Total |
|----------|----|----|----|----|-------|
| Error Handling | 5 | 2 | 0 | 0 | 7 |
| Code Quality | 6 | 8 | 0 | 0 | 14 |
| Type Design | 0 | 3 | 3 | 0 | 6 |
| Testing | 3 | 4 | 2 | 0 | 9 |
| Performance | 0 | 6 | 3 | 0 | 9 |
| Architecture | 0 | 5 | 1 | 0 | 6 |
| Dependencies | 1 | 2 | 4 | 0 | 7 |
| Documentation | 0 | 4 | 5 | 1 | 10 |
| Tooling | 1 | 2 | 1 | 1 | 5 |
| **TOTAL** | **16** | **36** | **19** | **2** | **73** |

## Effort Summary

| Priority | Total Hours | Average |
|----------|-------------|---------|
| P0 | 65 hours | 4.1 hours/task |
| P1 | 124 hours | 3.4 hours/task |
| P2 | 64 hours | 3.4 hours/task |
| P3 | 5 hours | 2.5 hours/task |
| **TOTAL** | **258 hours** | **3.5 hours/task** |

## Timeline Estimate (10 hours/week)

- **Q1 2026 (12 weeks, 120 hours)**: All P0 items + ~5 high-value P1 items
- **Q2 2026 (13 weeks, 130 hours)**: Remaining P1 items + key P2 items
- **H2 2026+ (26 weeks)**: P2/P3 items + ongoing maintenance

---

# EXECUTION GUIDELINES

## Getting Started (First Week)

1. **Day 1**: Review this plan (this document)
2. **Day 2**: Complete Tasks 1.1.1 (unwrap audit) & 2.1.1-2.1.6 (clippy fixes)
3. **Day 3**: Setup Task 9.1.1 (pre-commit hooks)
4. **Day 4**: Complete Tasks 2.3.1-2.3.2 (debug code cleanup)
5. **Day 5**: Create GitHub issues (Task 2.4.1)

## Running Tasks

### For Each Task:
1. **Understand**: Read task description fully
2. **Check Dependencies**: Ensure prerequisite tasks complete
3. **Create Branch**: `git checkout -b task-CATEGORY-NUMBER`
4. **Implement**: Follow task steps
5. **Test**: Run success criteria tests
6. **Review**: Self-review against criteria
7. **Commit**: Clear commit message: "Task 1.1.2: Replace high-risk unwraps"
8. **PR**: Create PR, reference task in description

### Task Completion Checklist:
```markdown
- [ ] Reviewed task description
- [ ] Checked dependencies complete
- [ ] Created feature branch
- [ ] Implemented changes
- [ ] All success criteria met
- [ ] Tests pass
- [ ] Code reviewed
- [ ] Committed and pushed
- [ ] PR created
```

## Dependency Management

**Key Rule**: Tasks have minimal dependencies. Most can be done in parallel.

**Recommended Parallization**:
- Team member A: Error handling (Section 1)
- Team member B: Code quality (Section 2)
- Team member C: Testing (Section 4)
- Team member D: Performance (Section 5) [Start after profiling]
- Team member E: Architecture (Section 6) [Start design first]

---

# SUCCESS METRICS

After completing all tasks, you should achieve:

✅ **Error Handling**:
- Unwrap calls: 584 → <100
- Panic calls: 13 → 0
- All error types documented

✅ **Code Quality**:
- Clippy warnings: 40+ → 0
- Largest files split to <2000 lines
- All TODOs completed or tracked

✅ **Testing**:
- Coverage: baseline → 70-80%
- Integration tests comprehensive
- Error scenarios tested

✅ **Performance**:
- Profiling baseline established
- Top hotspot optimized 20%+
- Benchmarks tracked

✅ **Architecture**:
- Event bus implemented
- UI/business logic separated
- Code more testable

✅ **Documentation**:
- All public APIs documented
- Architecture explained
- Contributing guide complete

---

**Total Effort**: ~258 hours across all tasks  
**Optimal Team**: 2-3 developers working in parallel  
**Recommended Timeline**: Q1-Q2 2026 (6 months)

Each task is designed to be independent and non-blocking, allowing maximum parallelization.

