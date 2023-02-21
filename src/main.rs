use egui_backend::egui::Window;
use egui_backend::{GfxBackend, UserApp, WindowBackend};
use egui_glow_glfw_emscripten::register_egui;
use egui_render_glow::GlowBackend;
use egui_window_glfw_passthrough::{GlfwBackend, GlfwConfig};
use mlua::{Function, Lua};

/// This is our userdata.
struct UserAppData<WB: WindowBackend> {
    /// lua code that we can edit live and execute.
    code: String,
    /// well.. lua vm to execute the above code.
    lua_vm: mlua::Lua,
    /// glow (rusty opengl wrapper) renderer to draw egui.. and other stuff with opengl. use three-d backend for high level features like gltf/meshes/materials/lighting etc..
    glow_backend: GlowBackend,
    /// egui context
    egui_context: egui_backend::egui::Context,
    window_backend: WB,
}
/// just some default lua code to show in text editor
pub const LUA_CODE: &str = r#"
function gui(ui)
	ui:label("just a label");
	local res = ui:button("lua butt");
	if res:clicked() then
		print("clicked button");
	end
end
"#;

// we care generic over the window backend. so, we can just decide at runtime which backend to use. eg: winit, glfw3, sdl2 are provided by `etk`
impl<WB: egui_backend::WindowBackend> UserApp for UserAppData<WB> {
    // these are used by some default trait method implementations to abstract out the common parts like providing egui input or drawing egui output etc..
    type UserGfxBackend = GlowBackend;

    type UserWindowBackend = WB;

    fn get_all(
        &mut self,
    ) -> (
        &mut Self::UserWindowBackend,
        &mut Self::UserGfxBackend,
        &egui_backend::egui::Context,
    ) {
        (
            &mut self.window_backend,
            &mut self.glow_backend,
            &self.egui_context,
        )
    }

    // the only function we care about. add whatever gui code you want.
    fn gui_run(&mut self) {
        let egui_context = self.egui_context.clone();

        Window::new("hello window").show(&egui_context, |ui| {
            ui.label(
                r#"
                This is where you write lua code. 
                1. there must be a function defined with the name `gui`
                2. the function will take a single argument (`&mut Ui`)
                3. you can use either label or button methods on that argument.
                4. button method returns a response object
                5. response has clicked method, which returns a bool (true if button is clicked).
                6. print statements go to console logs.
            "#,
            );
            ui.code_editor(&mut self.code);

            let res = ui.button("run code");
            if res.clicked() {
                if let Err(e) = self.lua_vm.load(&self.code).exec() {
                    eprintln!("failed to run lua code: {e}")
                }
            }
        });
        Window::new("Lua UI").show(&egui_context, |ui| {
            if let Ok(f) = self.lua_vm.globals().get::<_, Function>("gui") {
                let scope_result = self.lua_vm.scope(|scope| {
                    match scope.create_any_userdata_ref_mut(ui) {
                        Ok(uiud) => match f.call(uiud) {
                            Ok(()) => {}
                            Err(e) => {
                                tracing::warn!("failed to call gui fn due to error: {e}")
                            }
                        },
                        Err(e) => {
                            tracing::warn!("failed to send ui into lua due to error: {e}")
                            // ui.label("couldn't create ui userdata");
                        }
                    }
                    Ok(())
                });
                if scope_result.is_err() {
                    ui.label("couldn't create scope ");
                }
            } else {
                ui.label("gui fn doesn't exist yet");
            }
        });
        // this tells window to immediately repaint after vsync. otherwise, it will sleep, waiting for events -> causing unresponsive browser tab.
        egui_context.request_repaint();
    }
}
fn main() {
    // init logging
    tracing_subscriber::fmt().init();
    // just create a new backend. ask window backend for an opengl window because we chose glow backend. on vulkan/dx/metal(desktop), we would choose non-gl window.
    let mut window_backend = GlfwBackend::new(GlfwConfig::default(), Default::default());
    // create a opengl backend.
    let glow_backend = GlowBackend::new(&mut window_backend, Default::default());
    // window_backend.set_window_size([1200.0, 700.0]);
    let lua_vm = Lua::new();
    register_egui(&lua_vm);
    // create our app data
    let app = UserAppData {
        code: LUA_CODE.to_string(),
        lua_vm,
        glow_backend,
        egui_context: Default::default(),
        window_backend,
    };
    // enter event loop and run forever :)
    GlfwBackend::run_event_loop(app);
}
