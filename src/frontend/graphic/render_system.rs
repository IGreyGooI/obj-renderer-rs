use super::renderer::RendererState;
use std::sync::{
    RwLock, Arc,
};
use super::window::WindowState;
use gfx_hal::window::Extent2D;
use crate::frontend::graphic::constants::RENDER_SIZE;

pub struct Painter {
    renderer: Arc<RwLock<RendererState>>
}

impl Painter {
    pub fn new(
        window_state: &WindowState,
        render_size: Extent2D,
    ) -> Painter {
        let renderer = Arc::new(
                RwLock::new(
                    RendererState::new(
                        window_state,
                        RENDER_SIZE
                    )
                )
            );
        Painter {
            renderer,
        }
    }

}

