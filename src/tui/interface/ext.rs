use anyhow::Result;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::{
    context::Context,
    tui::{action::TuiAction, interface::types::TuiInterfaceType},
};

#[allow(async_fn_in_trait)]
pub trait TuiInterfaceExt {
    /// Retrieves the title of the interface (for the window).
    ///
    /// # Returns
    /// The title of the interface.
    fn title(&self) -> String;

    /// Whether or not this is a top-level interface (i.e., one that can be accessed rom anywhere in the app).
    ///
    /// # Returns
    /// True if this is a top-level interface, false otherwise.
    fn is_top_level(&self) -> bool;

    /// Retrieves the type of the interface.
    ///
    /// # Returns
    /// The type of the interface.
    fn get_type(&self) -> TuiInterfaceType;

    /// Retrieves the parent interface type, if any.
    ///
    /// # Returns
    /// An `Option` containing the parent interface type, or `None` if there is no parent.
    fn parent(&self) -> Option<TuiInterfaceType>;

    /// Prepares the interface for use, performing any necessary setup or initialization.
    ///
    /// # Arguments
    /// * `ctx` - The application context.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the preparation.
    async fn prepare(&mut self, ctx: Context) -> Result<()>;

    /// Cleans up the interface, performing any necessary teardown or resource release.
    ///
    /// # Arguments
    /// * `ctx` - The application context.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the cleanup.
    async fn cleanup(&mut self, ctx: Context) -> Result<()>;

    /// Retrieves the key bindings for the interface, providing a list of key-action pairs.
    ///
    /// # Returns
    /// A vector of tuples, where each tuple contains a key (as a string) and its corresponding action description (as a string).
    fn get_key_bindings(&self) -> Vec<(&str, &str)>;

    /// Draws the interface on the provided frame and area (the body of the application).
    ///
    /// # Arguments
    /// * `frame` - The frame to draw on.
    /// * `area` - The area of the frame to draw in (the body of the application).
    /// * `ctx` - The application context.
    fn draw<'a>(&self, frame: &mut Frame<'a>, area: Rect, ctx: Context);

    /// Handles user input for the interface, processing key events and performing corresponding actions.
    ///
    /// # Arguments
    /// * `ev` - The key event representing user input.
    /// * `ctx` - The application context.
    /// # Returns
    /// A `Result` containing a `TuiAction` indicating the action to be taken based on the input, or an error if the input handling fails.
    async fn handle_input(&mut self, ev: KeyEvent, ctx: Context) -> Result<TuiAction>;
}
