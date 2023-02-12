use egui_backend::egui::{Response, Ui};
use mlua::{Lua, UserDataMethods, UserDataRegistrar};

pub fn register_egui(lua: &Lua) {
    lua.register_userdata_type(|reg: &mut UserDataRegistrar<Response>| {
        reg.add_method_mut("clicked", |_, this, ()| Ok(this.clicked()));
    })
    .unwrap();
    lua.register_userdata_type(|reg: &mut UserDataRegistrar<Ui>| {
        reg.add_method_mut("label", |_, ui, s: String| {
            ui.label(&s);
            Ok(())
        });
        reg.add_method_mut("button", |lua, ui, s: String| {
            let res = ui.button(&s);
            let ud = lua.create_any_userdata(res).unwrap();
            Ok(ud)
        });
    })
    .unwrap();
}
