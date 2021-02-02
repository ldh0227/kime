use std::{convert::TryInto, num::NonZeroU32};

use x11rb::{
    protocol::xproto::{ConnectionExt, EventMask, KeyPressEvent, KEY_PRESS_EVENT},
    rust_connection::RustConnection,
};
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

use kime_engine_cffi::{
    Config, InputEngine, InputResultType, MODIFIER_CONTROL, MODIFIER_SHIFT, MODIFIER_SUPER,
};

pub struct KimeData {
    engine: InputEngine,
}

impl KimeData {
    pub fn new() -> Self {
        Self {
            engine: InputEngine::new(),
        }
    }
}

pub struct KimeHandler {
    config: Config,
    root: u32,
}

impl KimeHandler {
    pub fn new(root: u32, config: Config) -> Self {
        Self { config, root }
    }
}

impl KimeHandler {
    fn preedit(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        // FIXME: on-the-spot?

        let (x, y) = find_position(server.conn(), self.root, ic.app_win(), ic.preedit_spot())?;

        ic.user_data.engine.update_preedit(x, y, ch);

        Ok(())
    }

    fn reset(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        ic: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        if let Some(c) = ic.user_data.engine.reset() {
            self.clear_preedit(server, ic)?;
            self.commit(server, ic, c)?;
        }

        Ok(())
    }

    fn clear_preedit(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        ic: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        ic.user_data.engine.remove_preedit();
        Ok(())
    }

    fn commit(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        server.commit(ic, s)?;
        Ok(())
    }
}

impl ServerHandler<X11rbServer<RustConnection>> for KimeHandler {
    type InputStyleArray = [InputStyle; 3];
    type InputContextData = KimeData;

    fn new_ic_data(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        _input_style: InputStyle,
    ) -> Result<Self::InputContextData, xim::ServerError> {
        Ok(KimeData::new())
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            // over-spot
            InputStyle::PREEDIT_NOTHING | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NONE,
            // // on-the-spot when enable this java awt doesn't work I don't know why
        ]
    }

    fn handle_connect(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_set_ic_values(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::trace!("spot: {:?}", input_context.preedit_spot());

        if let Some(preedit) = input_context.user_data.engine.preedit_char() {
            self.clear_preedit(server, input_context)?;
            self.preedit(server, input_context, preedit)?;
        }

        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::info!(
            "IC created style: {:?}, spot_location: {:?}",
            input_context.input_style(),
            input_context.preedit_spot()
        );
        server.set_event_mask(input_context, EventMask::KEY_PRESS.into(), 0)?;

        Ok(())
    }

    fn handle_reset_ic(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<String, xim::ServerError> {
        log::trace!("reset_ic");

        Ok(input_context
            .user_data
            .engine
            .reset()
            .map(Into::into)
            .unwrap_or_default())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        // skip release
        if xev.response_type != KEY_PRESS_EVENT {
            return Ok(false);
        }

        log::trace!("{:?}", xev);

        // other modifiers then shift or lock or control or numlock or super
        if xev.state & !(0x1 | 0x2 | 0x4 | 0x10 | 0x40) != 0 {
            self.reset(server, input_context)?;
            return Ok(false);
        }

        let mut state = 0;

        if xev.state & 0x1 != 0 {
            state |= MODIFIER_SHIFT;
        }

        if xev.state & 0x4 != 0 {
            state |= MODIFIER_CONTROL;
        }

        if xev.state & 0x40 != 0 {
            state |= MODIFIER_SUPER;
        }

        let ret = input_context
            .user_data
            .engine
            .press_key(&self.config, xev.detail as u16, state);

        log::trace!("{:?}", ret);

        match ret.ty {
            InputResultType::Bypass => Ok(false),
            InputResultType::ToggleHangul => Ok(true),
            InputResultType::ClearPreedit => {
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitBypass => {
                self.commit(server, input_context, ret.char1)?;
                self.clear_preedit(server, input_context)?;
                Ok(false)
            }
            InputResultType::Commit => {
                self.commit(server, input_context, ret.char1)?;
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitCommit => {
                self.commit(server, input_context, ret.char1)?;
                self.commit(server, input_context, ret.char2)?;
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitPreedit => {
                self.commit(server, input_context, ret.char1)?;
                self.preedit(server, input_context, ret.char2)?;
                Ok(true)
            }
            InputResultType::Preedit => {
                self.preedit(server, input_context, ret.char1)?;
                Ok(true)
            }
        }
    }

    fn handle_destory_ic(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        _input_context: xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("destroy_ic");
        Ok(())
    }

    fn handle_preedit_start(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_caret(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
        _position: i32,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_set_focus(
        &mut self,
        _server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        input_context.user_data.engine.focus_in();
        Ok(())
    }

    fn handle_unset_focus(
        &mut self,
        server: &mut X11rbServer<RustConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        input_context.user_data.engine.focus_out();
        self.reset(server, input_context)
    }
}

fn find_position(
    conn: &RustConnection,
    root: u32,
    app_win: Option<NonZeroU32>,
    spot_location: xim::Point,
) -> Result<(u32, u32), xim::ServerError> {
    match app_win {
        Some(app_win) => {
            let offset = conn
                .translate_coordinates(app_win.get(), root, spot_location.x, spot_location.y)?
                .reply()?;

            Ok((
                offset.dst_x.try_into().unwrap_or_default(),
                offset.dst_y.try_into().unwrap_or_default(),
            ))
        }
        _ => Ok((0, 0)),
    }
}
