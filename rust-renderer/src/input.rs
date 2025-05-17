use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, console};
use std::cell::RefCell;
use std::rc::Rc;


#[wasm_bindgen]
pub fn handle_mouse_click(x: f64, y: f64) {
    console::log_1(&format!("INFO: mouse click at: {}, {}", x, y).into());
}

pub fn enable_mouse_controls(
    canvas: HtmlCanvasElement,
    rotation: Rc<RefCell<(f64, f64)>>,
) -> Result<(), JsValue> {
    let canvas = Rc::new(canvas);
    let is_dragging = Rc::new(RefCell::new(false));
    let last_mouse_pos = Rc::new(RefCell::new((0.0, 0.0)));

    // Clone references for the `mousedown` event
    let canvas_clone = canvas.clone();
    let is_dragging_clone = is_dragging.clone();
    let last_mouse_pos_clone = last_mouse_pos.clone();

    // Mouse down event
    let on_mouse_down = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        *is_dragging_clone.borrow_mut() = true;
        *last_mouse_pos_clone.borrow_mut() = (event.client_x() as f64, event.client_y() as f64);
    }) as Box<dyn FnMut(_)>);
    canvas_clone
        .add_event_listener_with_callback("mousedown", on_mouse_down.as_ref().unchecked_ref())?;
    on_mouse_down.forget();

    // Clone references for the `mousemove` event
    let canvas_clone = canvas.clone();
    let is_dragging_clone = is_dragging.clone();
    let last_mouse_pos_clone = last_mouse_pos.clone();
    let rotation_clone = rotation.clone();

    // Mouse move event
    let on_mouse_move = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if *is_dragging_clone.borrow() {
            let (last_x, last_y) = *last_mouse_pos_clone.borrow();
            let (current_x, current_y) = (event.client_x() as f64, event.client_y() as f64);

            // Calculate the change in mouse position
            let delta_x = current_x - last_x;
            let delta_y = current_y - last_y;

            // Update rotation angles (scale the deltas for smoother rotation)
            let mut rotation = rotation_clone.borrow_mut();
            rotation.0 += delta_y * 0.05; // Rotate around X-axis
            rotation.1 += delta_x * 0.05; // Rotate around Y-axis

            // Update the last mouse position
            *last_mouse_pos_clone.borrow_mut() = (current_x, current_y);

            //web_sys::console::log_1(&format!("Rotation {},{}", rotation.0, rotation.1).into());
        }
    }) as Box<dyn FnMut(_)>);
    canvas_clone
        .add_event_listener_with_callback("mousemove", on_mouse_move.as_ref().unchecked_ref())?;
    on_mouse_move.forget();

    // Clone references for the `mouseup` event
    let is_dragging_clone = is_dragging.clone();

    // Mouse up event
    let on_mouse_up = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        *is_dragging_clone.borrow_mut() = false;
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mouseup", on_mouse_up.as_ref().unchecked_ref())?;
    on_mouse_up.forget();

    Ok(())
}
