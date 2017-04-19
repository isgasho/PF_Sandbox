use winit::VirtualKeyCode;

use controller::{Controller, ElementButton, ElementStick, ElementTrigger};
use input::Input;
use std::cmp;

pub enum Flow {
    Stop,
    Play,
    Rewind,
    Replay,
    StepFrames   (u64),
    RewindFrames (u64),
    ReplayFrames (u64),
}

pub struct State {
    pub controllers:             Vec<Controller>,
    pub current_controller:      usize,
    pub display_all_controllers: bool,
    pub use_aspect_ratio:        bool,
    pub display_analog_as_float: bool,
    pub touchtype:               bool,
    pub flow:                    Flow,
    number:                      NumberInput,
}

impl State {
    pub fn new() -> State {
        State {
            controllers:             vec!(Controller::new()),
            current_controller:      0,
            display_all_controllers: false,
            use_aspect_ratio:        false,
            display_analog_as_float: false,
            touchtype:               true,
            flow:                    Flow::Stop,
            number:                  NumberInput::new(),
        }
    }

    pub fn update(&mut self, input: &Input) {
        // toggle between display all controllers / display selected controller
        if input.key_pressed(VirtualKeyCode::Q) {
            self.display_all_controllers = !self.display_all_controllers;
        }

        // Toggle render graphics to aspect_ratio / stretch to fill
        if input.key_pressed(VirtualKeyCode::W) {
            self.use_aspect_ratio = !self.use_aspect_ratio;
        }

        // toggle display float values / byte values for sticks and triggers
        if input.key_pressed(VirtualKeyCode::E) {
            self.display_analog_as_float = !self.display_analog_as_float;
        }

        // toggle touch typing mode and 1-1 keybindings
        if input.key_pressed(VirtualKeyCode::R) {
            self.touchtype = !self.touchtype
        }

        // controller select
        if input.key_pressed(VirtualKeyCode::F1) && self.controllers.len() > 0 {
            self.current_controller = 0;
        }
        else if input.key_pressed(VirtualKeyCode::F2) && self.controllers.len() > 1 {
            self.current_controller = 1;
        }
        else if input.key_pressed(VirtualKeyCode::F3) && self.controllers.len() > 2 {
            self.current_controller = 2;
        }
        else if input.key_pressed(VirtualKeyCode::F4) && self.controllers.len() > 3 {
            self.current_controller = 3;
        }
        else if input.key_pressed(VirtualKeyCode::F5) && self.controllers.len() > 4 {
            self.current_controller = 4;
        }
        else if input.key_pressed(VirtualKeyCode::F6) && self.controllers.len() > 5 {
            self.current_controller = 5;
        }
        else if input.key_pressed(VirtualKeyCode::F7) && self.controllers.len() > 6 {
            self.current_controller = 6;
        }
        else if input.key_pressed(VirtualKeyCode::F8) && self.controllers.len() > 7 {
            self.current_controller = 7;
        }
        else if input.key_pressed(VirtualKeyCode::F9) && self.controllers.len() > 8 {
            self.current_controller = 8;
        }
        else if input.key_pressed(VirtualKeyCode::F10) && self.controllers.len() > 9 {
            self.current_controller = 9;
        }
        else if input.key_pressed(VirtualKeyCode::F11) && self.controllers.len() > 10 {
            self.current_controller = 10;
        }
        else if input.key_pressed(VirtualKeyCode::F12) && self.controllers.len() > 11 {
            self.current_controller = 11;
        }

        // add/remove controllers
        if input.key_pressed(VirtualKeyCode::LBracket) {
            if self.controllers.len() > 1 {
                self.controllers.pop();
            }
        }
        else if input.key_pressed(VirtualKeyCode::RBracket) {
            if self.controllers.len() < 9 {
                self.controllers.push(Controller::new());
            }
        }

        // number input
        if input.key_pressed(VirtualKeyCode::Key0) {
            self.number.input(0);
        }
        else if input.key_pressed(VirtualKeyCode::Key1) {
            self.number.input(1);
        }
        else if input.key_pressed(VirtualKeyCode::Key2) {
            self.number.input(2);
        }
        else if input.key_pressed(VirtualKeyCode::Key3) {
            self.number.input(3);
        }
        else if input.key_pressed(VirtualKeyCode::Key4) {
            self.number.input(4);
        }
        else if input.key_pressed(VirtualKeyCode::Key5) {
            self.number.input(5);
        }
        else if input.key_pressed(VirtualKeyCode::Key6) {
            self.number.input(6);
        }
        else if input.key_pressed(VirtualKeyCode::Key7) {
            self.number.input(7);
        }
        else if input.key_pressed(VirtualKeyCode::Key8) {
            self.number.input(8);
        }
        else if input.key_pressed(VirtualKeyCode::Key9) {
            self.number.input(9);
        }
        else if input.key_pressed(VirtualKeyCode::Subtract) {
            self.number.negative();
        }
        else if input.key_pressed(VirtualKeyCode::Equals) {
            self.number.positive();
        }

        // Key -> GC mapping
        let controller = &mut self.controllers[self.current_controller];

        State::map_button(input, VirtualKeyCode::Up,    &mut controller.up);
        State::map_button(input, VirtualKeyCode::Down,  &mut controller.down);
        State::map_button(input, VirtualKeyCode::Left,  &mut controller.left);
        State::map_button(input, VirtualKeyCode::Right, &mut controller.right);

        State::map_button(input, VirtualKeyCode::A, &mut controller.a);
        State::map_button(input, VirtualKeyCode::S, &mut controller.b);
        State::map_button(input, VirtualKeyCode::D, &mut controller.x);
        State::map_button(input, VirtualKeyCode::F, &mut controller.y);

        State::map_button(input, VirtualKeyCode::G, &mut controller.start);
        State::map_button(input, VirtualKeyCode::H, &mut controller.z);

        State::map_button (input, VirtualKeyCode::J,     &mut controller.l);
        State::map_trigger(input, VirtualKeyCode::K,     &mut controller.l_trigger, &mut self.number);
        State::map_trigger(input, VirtualKeyCode::L,     &mut controller.r_trigger, &mut self.number);
        State::map_button (input, VirtualKeyCode::Colon, &mut controller.r);

        State::map_stick(input, VirtualKeyCode::Y, &mut controller.stick_x,   &mut self.number);
        State::map_stick(input, VirtualKeyCode::U, &mut controller.stick_y,   &mut self.number);
        State::map_stick(input, VirtualKeyCode::I, &mut controller.c_stick_x, &mut self.number);
        State::map_stick(input, VirtualKeyCode::O, &mut controller.c_stick_y, &mut self.number);

        // Game flow
        if input.key_pressed(VirtualKeyCode::Return) {
            self.flow = match self.flow {
                Flow::Play => { Flow::Stop }
                _          => { Flow::Play }
            };
        }
        else if input.key_pressed(VirtualKeyCode::Space) {
            self.flow = Flow::StepFrames(self.number.pop_frames());
        }
        else if input.key_pressed(VirtualKeyCode::Z) {
            self.flow = Flow::RewindFrames(self.number.pop_frames());
        }
        else if input.key_pressed(VirtualKeyCode::X) {
            self.flow = Flow::ReplayFrames(self.number.pop_frames());
        }
        else if input.key_pressed(VirtualKeyCode::C) {
            self.flow = Flow::Rewind;
        }
        else if input.key_pressed(VirtualKeyCode::V) {
            self.flow = Flow::Replay;
        }
    }

    fn map_button(input: &Input, key: VirtualKeyCode, button: &mut ElementButton) {
        if input.key_pressed(key) {
            if input.held_shift() {
                button.hold();
            }
            else {
                button.press();
            }
        }
    }

    fn map_stick(input: &Input, key: VirtualKeyCode, button: &mut ElementStick, number: &mut NumberInput) {
        if input.key_pressed(key) {
            if input.held_shift() {
                button.hold(number.pop_stick());
            }
            else {
                button.press(number.pop_stick());
            }
        }
    }

    fn map_trigger(input: &Input, key: VirtualKeyCode, button: &mut ElementTrigger, number: &mut NumberInput) {
        if input.key_pressed(key) {
            if input.held_shift() {
                button.hold(number.pop_trigger());
            }
            else {
                button.press(number.pop_trigger());
            }
        }
    }
}

struct NumberInput {
    value:    u64,
    negative: bool,
}

impl NumberInput {
    pub fn new() -> NumberInput {
        NumberInput {
            value: 0,
            negative: false,
        }
    }

    // setters

    pub fn input(&mut self, input: u64) {
        self.value = self.value.saturating_mul(10).saturating_add(input);
    }

    pub fn negative(&mut self) {
        self.value = 0;
        self.negative = true;
    }

    pub fn positive(&mut self) {
        self.value = 0;
        self.negative = false;
    }

    // users

    pub fn pop_stick(&mut self) -> i8 {
        let value_i8 = cmp::min(self.value, i8::max_value() as u64) as i8;
        self.value = 0;
        self.negative = false;
        (value_i8).saturating_mul(if self.negative { -1 } else { 1 })
    }

    pub fn pop_trigger(&mut self) -> u8 {
        let value_u8 = cmp::min(self.value, u8::max_value() as u64) as u8;
        self.value = 0;
        self.negative = false;
        value_u8
    }

    pub fn pop_frames(&mut self) -> u64 {
        let result = self.value;
        self.value = 0;
        if result == 0 {
            1
        } else {
            result
        }
    }
}
