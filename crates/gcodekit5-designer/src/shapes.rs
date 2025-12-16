//! Geometric shapes for the designer tool.
//! Deprecated: Use crate::model instead.

/// Type of CAM operation to perform on the shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Profile,
    Pocket,
}

impl Default for OperationType {
    fn default() -> Self {
        Self::Profile
    }
}
