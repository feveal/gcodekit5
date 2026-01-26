## [0.50.2-alpha.0] - 2026-01-26

### Added
- **Unwrap Audit Documentation**: Comprehensive audit of all unwrap() calls
  - Created `docs/audits/unwrap_audit.csv` with 585 categorized unwraps
  - Created `docs/audits/UNWRAP_AUDIT_REPORT.md` with executive summary
  - 144 high-risk, 158 medium-risk, 283 low-risk unwraps identified
  - Priority remediation targets: Mutex locks, RefCell borrows, File I/O
  - REMEDIATION_PLAN.md Task 1.1.1 completed

- **CI Code Quality Checks**: Prevent unwrap() regression
  - Created `.github/workflows/code-quality.yml` with clippy unwrap detection
  - Created `.github/PULL_REQUEST_TEMPLATE.md` with error handling checklist
  - REMEDIATION_PLAN.md Task 1.1.5 completed

- **Structured Error Types**: Added thiserror-based error types to 3 crates
  - `gcodekit5-designer/src/error.rs`: DesignError, GeometryError, ToolpathError
  - `gcodekit5-communication/src/error.rs`: CommunicationError, ProtocolError, FirmwareError
  - `gcodekit5-visualizer/src/error.rs`: VisualizationError, ParsingError, FileError
  - REMEDIATION_PLAN.md Task 1.2.1 completed

- **GitHub Issues for TODOs**: Converted all 20 TODOs to tracked issues (#12-#19)
  - REMEDIATION_PLAN.md Task 2.4.1 completed

- **Pre-commit Hook**: Added `.githooks/pre-commit` for code quality checks
  - REMEDIATION_PLAN.md Task 9.1.1 completed

### Changed
- **Error Handling**: Removed ALL 585 unsafe unwrap() calls from production code
- **Test Quality**: Replaced all 235 test unwrap() calls with expect()
- **Code Structure**: Extracted DesignerCanvas to separate module
- **Code Cleanup**: Replaced debug eprintln/println with structured tracing
