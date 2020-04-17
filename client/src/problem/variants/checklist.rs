use common::problems;
use log::*;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};

pub enum ChecklistMsg {
    ChoiceSelected(usize),
}

#[derive(Debug, Clone, Properties)]
pub struct ChecklistProps {
    pub options: Vec<String>,
    pub solution: Vec<usize>,
}

pub struct ChecklistComponent {
    link: ComponentLink<Self>,
    props: ChecklistProps,
    choices: Vec<bool>,
}

impl ChecklistComponent {
    pub fn select_choice(&self, idx: usize) -> Callback<MouseEvent> {
        self.link
            .callback(move |_| ChecklistMsg::ChoiceSelected(idx))
    }
}

impl Component for ChecklistComponent {
    type Message = ChecklistMsg;
    type Properties = ChecklistProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let length = props.options.len();
        Self {
            link,
            props,
            choices: vec![false; length],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            ChecklistMsg::ChoiceSelected(idx) => self.choices[idx] = !self.choices[idx],
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="checklist">
                {
                    for self.props.options.iter().enumerate().map(|(i, option)| html! {
                        <div class="optionwrapper">
                            <span class="optionmarker">{ ">" }</span>
                            <div onclick=self.select_choice(i) class={
                                if self.choices[i] {
                                    "option selected"
                                } else {
                                    "option"
                                }
                            }>{option}</div>
                        </div>
                    })
                }
            </div>
        }
    }
}
