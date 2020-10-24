macro_rules! inject_wasm_test_boilerplate
{
    () =>
    {
        use wasm_bindgen_test::*;
        wasm_bindgen_test_configure!(run_in_browser);
        use wasm_bindgen::
        {
            prelude::*,
            JsCast,
        };
        use web_sys::*;
        use crate::gfx::
        {
            Context,
            GfxError,
            GlError,
            gl_get_errors,
        };

        fn get_context() -> Context
        {
            let window: Window = window().expect("window context");
            let document: Document = window.document().expect("document context");

            let canvas =
                {
                    let elem = document.create_element("CANVAS").expect("new canvas element");
                    elem.set_id("canvas");
                    document.body().expect("document body").append_child(&elem).expect("canvas added to body");
                    elem.dyn_into::<HtmlCanvasElement>().expect("cast canvas element")
                };
            crate::gfx::new_context(&canvas).expect("context")
        }
    }
}
