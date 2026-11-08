//! Handling of xdg dialog, which is used to mark a toplevel as a modal dialog.

use sctk::globals::GlobalData;
use sctk::reexports::client::globals::{BindError, GlobalList};
use sctk::reexports::client::{delegate_dispatch, Connection, Dispatch, Proxy, QueueHandle};
use sctk::reexports::protocols::xdg::dialog::v1::client::xdg_dialog_v1::XdgDialogV1;
use sctk::reexports::protocols::xdg::dialog::v1::client::xdg_wm_dialog_v1::XdgWmDialogV1;
use sctk::reexports::protocols::xdg::shell::client::xdg_toplevel::XdgToplevel;

use crate::error::NotSupportedError;
use crate::platform_impl::wayland::state::WinitState;

pub struct XdgDialogState {
    xdg_wm_dialog: XdgWmDialogV1,
}

impl XdgDialogState {
    pub fn bind(
        globals: &GlobalList,
        queue_handle: &QueueHandle<WinitState>,
    ) -> Result<Self, BindError> {
        let xdg_wm_dialog = globals.bind(queue_handle, 1..=1, GlobalData)?;
        Ok(Self { xdg_wm_dialog })
    }

    pub fn global(&self) -> &XdgWmDialogV1 {
        &self.xdg_wm_dialog
    }
}

/// The `xdg_dialog_v1` of one toplevel, created the first time the toplevel is made modal.
pub struct ToplevelDialog {
    /// [`None`] if the compositor does not offer `xdg_wm_dialog_v1`.
    manager: Option<XdgWmDialogV1>,
    dialog: Option<XdgDialogV1>,
}

impl ToplevelDialog {
    pub fn new(manager: Option<XdgWmDialogV1>) -> Self {
        Self { manager, dialog: None }
    }

    pub fn set_modal(
        &mut self,
        toplevel: &XdgToplevel,
        modal: bool,
        queue_handle: &QueueHandle<WinitState>,
    ) -> Result<(), NotSupportedError> {
        let Some(manager) = &self.manager else { return Err(NotSupportedError::new()) };

        let dialog = self
            .dialog
            .get_or_insert_with(|| manager.get_xdg_dialog(toplevel, queue_handle, GlobalData));
        if modal {
            dialog.set_modal();
        } else {
            dialog.unset_modal();
        }
        Ok(())
    }
}

impl Drop for ToplevelDialog {
    fn drop(&mut self) {
        if let Some(dialog) = self.dialog.take() {
            dialog.destroy();
        }
    }
}

impl Dispatch<XdgWmDialogV1, GlobalData, WinitState> for XdgDialogState {
    fn event(
        _state: &mut WinitState,
        _proxy: &XdgWmDialogV1,
        _event: <XdgWmDialogV1 as Proxy>::Event,
        _data: &GlobalData,
        _conn: &Connection,
        _qhandle: &QueueHandle<WinitState>,
    ) {
    }
}

impl Dispatch<XdgDialogV1, GlobalData, WinitState> for ToplevelDialog {
    fn event(
        _state: &mut WinitState,
        _proxy: &XdgDialogV1,
        _event: <XdgDialogV1 as Proxy>::Event,
        _data: &GlobalData,
        _conn: &Connection,
        _qhandle: &QueueHandle<WinitState>,
    ) {
    }
}

delegate_dispatch!(WinitState: [ XdgWmDialogV1: GlobalData] => XdgDialogState);
delegate_dispatch!(WinitState: [ XdgDialogV1: GlobalData] => ToplevelDialog);
