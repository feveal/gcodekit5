pub mod manager;
pub mod model;
pub mod traits;
pub mod ui_integration;

pub use manager::DeviceManager;
pub use model::{AxisLimits, ControllerType, DeviceProfile, DeviceType};
pub use traits::DeviceProfileProvider;
pub use ui_integration::{DeviceProfileUiModel, DeviceUiController};
