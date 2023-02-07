use gloo::console::log;
use gloo::timers::callback::Timeout;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use wasm_bindgen::Clamped;
// use gloo_utils::window;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_sys::{HtmlInputElement, ImageData};
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::types::JSTimer;
use crate::types::Settings;
use crate::wfc_field::Cell;
use crate::wfc_field::{Pixel, WFCField};
use crate::worker::{Worker, WorkerInput, WorkerOutput};

const NUM_WORKERS: u8 = 5;

pub enum Msg {
    Draw,
    Epochs,
    Gen,
    WorkerStart,
    WorkerMsg(WorkerOutput),
    StartTimeout,
    StopTimeout,
    UpdateSettings,
}

pub struct Canvas {
    canvas: NodeRef,
    settings: Settings,
    field: Option<WFCField>,
    timer: JSTimer,
    workers: Box<[Box<dyn Bridge<Worker>>]>,
    workers_results: Vec<WorkerOutput>,
    timeout: Option<Timeout>,
    settings_nodes: [NodeRef; 2],
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        let settings = (200, 3);
        // let mut field = WFCField::new(settings.0);
        // field.init();

        let workers = (0..NUM_WORKERS)
            .map(|_| {
                let cb = {
                    let link = _ctx.link().clone();
                    move |e| link.send_message(Self::Message::WorkerMsg(e))
                };
                Worker::bridge(Rc::new(cb))
            })
            .collect::<Box<[Box<dyn Bridge<Worker>>]>>();

        Self {
            canvas: NodeRef::default(),
            settings,
            field: None,
            timer: JSTimer::new(),
            workers,
            timeout: None,
            workers_results: vec![],
            settings_nodes: [NodeRef::default(), NodeRef::default()],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Draw => {
                self.render_canvas();
                false
            }
            Msg::UpdateSettings => {
                ctx.link().send_message(Msg::StopTimeout);
                let dim: usize = self.settings_nodes[0].cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
                let scale: usize = self.settings_nodes[1].cast::<HtmlInputElement>().unwrap().value().parse().unwrap();
                self.settings = (dim, scale);
                ctx.link().send_message(Msg::WorkerStart);
                log!("Settings updated");
                false
            }
            Msg::Epochs => {
                // log!("Epochs start");
                // self.timer.start_time();
                // self.field.epoch();
                // self.timer.epoch_from_start("Epoch took");

                // log!("Epochs start");
                self.timer.start_time();
                let field = self.field.as_mut().unwrap();
                field.epoch3();
                self.timer.epoch_from_start("Epoch took");

                // self.start_epoch();

                // self.timer.start_time();
                ctx.link().send_message(Msg::Draw);
                // self.timer.epoch_from_start("Draw took");

                if field.collapsed_cnt >= field.len() - 1 {
                    log!("We break");
                    return false;
                }
                ctx.link().send_message(Msg::StartTimeout);
                false
            }
            Msg::Gen => {
                self.timer.start_time();
                for i in 0..500 {
                    // log!("Iter: ", i);
                    self.field.as_mut().unwrap().epoch3();
                }
                self.timer.epoch_from_start("Gen took");
                ctx.link().send_message(Msg::Draw);
                false
            }
            Msg::WorkerStart => {
                log!("Field loading");
                self.start_field_workers();
                false
            }
            Msg::WorkerMsg(v) => {
                self.workers_results.push(v);
                if self.workers_results.len() == NUM_WORKERS as usize {
                    self.join_workers();
                    ctx.link().send_message(Msg::Draw);
                }
                // self.field = Some(WFCField::new_with_data(v.value, self.settings.0));
                // ctx.link().send_message(Msg::Draw);
                log!("Worker done");
                false
            }
            Msg::StartTimeout => {
                let handle = {
                    let link = ctx.link().clone();
                    Timeout::new(3, move || link.send_message(Msg::Epochs))
                };
                self.timeout = Some(handle);

                false
            }
            Msg::StopTimeout => {
                self.timeout = None;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().batch_callback(move |_| vec![Msg::Epochs]);
        let onclick2 = ctx.link().callback(move |_| Msg::StopTimeout);
        let onclick3 = ctx.link().callback(move |_| Msg::Gen);

        let on_settings_change = ctx.link().callback(move |_| Msg::UpdateSettings);
        // ctx.link().send_message(Msg::Draw);
        ctx.link().send_message(Msg::WorkerStart);
        html! {
            <div>
                <button onclick={&onclick}>{"Start"}</button>
                <button onclick={&onclick2}>{"Stop"}</button>
                <button onclick={&onclick3}>{"Gen"}</button>

                <div>
                    <label for="dim">{"Dim (between 1 and 5000):"}
                        <input type="number" id="dim" name="dim" min="1" max="5000"
                        value={self.settings.0.to_string()}
                         onchange={&on_settings_change} ref={self.settings_nodes[0].clone()}
                         />
                    </label>
                </div>
                <div>
                    <label for="scale">{"Scale (between 1 and 10):"}
                        <input type="number" id="scale" name="scale" min="1" max="10" 
                        value={self.settings.1.to_string()}
                        onchange={&on_settings_change} ref={self.settings_nodes[1].clone()}
                        />
                    </label>
                </div>
                // <div>
                //     <label for="upper">{"Threshold"}
                //     <input type="range" min="0" max="256" class="slider" id="upper" onchange={&on_change} ref={self.input[0].clone()}/>
                //     </label>
                // </div>
                // <div>
                //     <label for="vertical">{"Vertical"}
                //     <input type="checkbox" id="vertical" onchange={&on_change} ref={self.input[1].clone()}/>
                //     </label>
                // </div>

                <div>
                    <canvas
                        id="canvas"
                        width={1000}
                        height={1000}
                        ref={self.canvas.clone()}>
                    </canvas>
                </div>
            </div>
        }
    }
}

impl Canvas {
    fn render_canvas(&self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let ctxx: CanvasRenderingContext2d =
            canvas.get_context("2d").unwrap().unwrap().unchecked_into();

        let scale = self.settings.1;
        let minus = 0;
        let field = self.field.as_ref().unwrap();
        ctxx.clear_rect(0.0, 0.0, 1000.0, 1000.0);
        for x in 0..field.dim {
            for y in 0..field.dim {
                let cl = &field.data[x * field.dim + y];
                let px = &cl.px;
                let cd = format!(
                    "rgba({},{},{},{})",
                    px.rgba[0], px.rgba[1], px.rgba[2], px.rgba[3]
                );

                ctxx.set_fill_style(&cd.into());
                ctxx.stroke();
                ctxx.fill_rect(
                    (x * scale) as f64,
                    (y * scale) as f64,
                    (scale - minus) as f64,
                    (scale - minus) as f64,
                );
            }
        }

        // --------------------------------------------------------------
        // let dim = self.field.dim;
        // let mut buffer = vec![];
        // for cell in self.field.data.iter() {
        //     buffer.extend(cell.px.rgba);
        // }

        // // log!(format!("{:?}", buffer));
        // let arr = Clamped(buffer.as_slice());
        // let img = ImageData::new_with_u8_clamped_array_and_sh(arr, dim as u32, dim as u32).unwrap();

        // ctxx.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        // canvas.set_width(dim as u32);
        // canvas.set_height(dim as u32);
        // let res = ctxx.put_image_data(&img, 1.0, 1.0);

        // log!(format!("{:?}", res));
    }

    fn start_field_workers(&mut self) {
        let n = NUM_WORKERS as usize;
        let dim = self.settings.0;
        let len = dim * dim;
        let step = len / n;
        let last_step = step + (len - (n * step));

        for i in 0..n {
            let worker = &mut self.workers[i];
            worker.send(WorkerInput {
                idx: i,
                len: if i == n - 1 { last_step } else { step },
                dim: self.settings.0,
            });
            log!(format!("Worker {} started", &i));
        }
    }

    fn join_workers(&mut self) {
        self.workers_results.sort_by(|a, b| a.idx.cmp(&b.idx));
        let data = self
            .workers_results
            .iter()
            .map(|x| x.value.clone().into_vec())
            .flatten()
            .collect::<Box<[Cell]>>();
        self.field = Some(WFCField::new_with_data(data, self.settings.0));

        self.workers_results = vec![];
        log!("Field loaded");
    }
}
