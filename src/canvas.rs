use gloo::console::log;
use gloo::timers::callback::Timeout;
use wasm_bindgen::Clamped;
use std::ops::Range;
use std::rc::Rc;
// use gloo_utils::window;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_sys::{HtmlInputElement, ImageData};
use yew_agent::{Bridge, Bridged};
use yew::prelude::*;

use crate::types::Settings;
use crate::types::JSTimer;
use crate::wfc_field::{Pixel, WFCField};
use crate::worker::{Worker, WorkerInput, WorkerOutput};


const NUM_WORKERS:u8 = 2;

pub enum Msg {
    Draw,
    Epochs,
    WorkerStart,
    WorkerMsg(WorkerOutput),
    StartTimeout,
    StopTimeout,
}

pub struct Canvas {
    canvas: NodeRef,
    settings: Settings,
    field: WFCField,
    timer: JSTimer,
    workers: Box<[Box<dyn Bridge<Worker>>]>,
    timeout: Option<Timeout>,
}

impl Component for Canvas {
    type Message = Msg;
    type Properties = ();
    fn create(_ctx: &Context<Self>) -> Self {
        let settings = (300,);
        let mut field = WFCField::new(settings.0);
        // field.init();

        let workers = (0..NUM_WORKERS).map(|_| {
            let cb = {
                let link = _ctx.link().clone();
                move |e| link.send_message(Self::Message::WorkerMsg(e))
            };
            Worker::bridge(Rc::new(cb))

        }).collect::<Box<[Box<dyn Bridge<Worker>>]>>();

        Self {
            canvas: NodeRef::default(),
            settings,
            field,
            timer: JSTimer::new(),
            workers,
            timeout: None
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Draw => {
                self.render_canvas();
                false
            
            }
            Msg::Epochs => {
                // log!("Epochs start");
                // self.timer.start_time();
                // self.field.epoch();
                // self.timer.epoch_from_start("Epoch took");

                // log!("Epochs start");
                // self.timer.start_time();
                self.field.epoch3();
                // self.timer.epoch_from_start("Epoch took");

                // self.start_epoch();

                // self.timer.start_time();
                ctx.link().send_message(Msg::Draw);
                // self.timer.epoch_from_start("Draw took");
                ctx.link().send_message(Msg::StartTimeout);
                false
            }
            Msg::WorkerStart => {
                // self.worker.send(WorkerInput {
                //     n: 5 as u32,
                // });
                false
            }
            Msg::WorkerMsg(v) => {
                log!(format!("Fibonacci value: {}", v.value));
                false
            }
            Msg::StartTimeout => {
                let handle = {
                    let link = ctx.link().clone();
                    Timeout::new(10, move || link.send_message(Msg::Epochs))
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
        ctx.link().send_message(Msg::Draw);
        html! {
            <div>
                <button onclick={&onclick}>{"Start"}</button>
                <button onclick={&onclick2}>{"Stop"}</button>
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
    fn start_epoch(&mut self) {
        if self.field.epoch_idx < 10 {
            self.field.epoch();
        }

        // get visited length
        // split into equal parts by NUM_WORKERS
        // pass visited into worker
        // 
    }

    fn render_canvas(&self) {
        let canvas: HtmlCanvasElement = self.canvas.cast().unwrap();
        let ctxx: CanvasRenderingContext2d =
            canvas.get_context("2d").unwrap().unwrap().unchecked_into();

        let scale = 3;
        let minus = 0;
        ctxx.clear_rect(0.0, 0.0, 1000.0, 1000.0);
        for x in 0..self.field.dim {
            for y in 0..self.field.dim {
                let cl = &self.field.data[x * self.field.dim + y];
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
}
