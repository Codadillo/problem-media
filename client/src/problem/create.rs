use crate::app::{AppRoute, API_URL};
use common::problems::{NewProblem, ProblemContent, ProblemType, Topic};
use log::*;
use serde::{Deserialize, Serialize};
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::{FetchTask, Request, Response},
        FetchService,
    },
};
use yew_router::{agent::RouteRequest, prelude::*};

#[derive(Debug, Clone)]
struct ProblemBuilder {
    topic: Option<Topic>,
    tags: Vec<String>,
    prompt: String,
    content: ProblemContentBuilder,
}

impl Default for ProblemBuilder {
    fn default() -> Self {
        Self {
            topic: Option::default(),
            tags: Vec::default(),
            prompt: String::default(),
            content: ProblemContentBuilder::default_from_type(&ProblemType::MultipleChoice),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProblemContentBuilder {
    MultipleChoice {
        options: Vec<String>,
        solution: Option<usize>,
    },
}

impl ProblemContentBuilder {
    fn get_type(&self) -> ProblemType {
        match self {
            ProblemContentBuilder::MultipleChoice { options, solution } => {
                ProblemType::MultipleChoice
            }
        }
    }
}

impl ProblemContentBuilder {
    fn default_from_type(p_type: &ProblemType) -> Self {
        match p_type {
            ProblemType::MultipleChoice => ProblemContentBuilder::MultipleChoice {
                options: vec![],
                solution: None,
            },
            _ => unimplemented!(),
        }
    }
}

pub enum CreateMsg {
    NoOp,
    // All problems
    SetTopic(Topic),
    SetContent(ProblemContentBuilder),
    UpdateTagText(String),
    RemoveTag(usize),
    CreateTag,
    UpdatePrompt(String),
    Finish,
    // Multiple choice & free response
    AddSolution(usize),
    UpdateChoice(String),
    AddChoice,
    RemoveChoice(usize),
    // Requests stuff
    CreationSuccess(i32),
    CreationFailure(String),
}

#[derive(Debug, Clone, Properties)]
pub struct CreateProps {
    pub user_id: i32,
}

pub struct CreateComponent {
    link: ComponentLink<Self>,
    props: CreateProps,
    router: Box<dyn Bridge<RouteAgent>>,
    fetch_service: FetchService,
    ft: Option<FetchTask>,
    builder: ProblemBuilder,
    new_tag_text: String,
    content_input_buffer: String,
    error_message: String,
}

impl CreateComponent {
    fn set_topic(&self, topic: Topic) -> Callback<MouseEvent> {
        self.link
            .callback(move |_| CreateMsg::SetTopic(topic.clone()))
    }

    fn update_tag_text(&self) -> Callback<InputData> {
        self.link
            .callback(move |data: InputData| CreateMsg::UpdateTagText(data.value))
    }

    fn remove_tag(&self, idx: usize) -> Callback<MouseEvent> {
        self.link.callback(move |_| CreateMsg::RemoveTag(idx))
    }

    fn create_tag(&self) -> Callback<MouseEvent> {
        self.link.callback(move |_| CreateMsg::CreateTag)
    }

    fn update_prompt(&self) -> Callback<InputData> {
        self.link
            .callback(move |data: InputData| CreateMsg::UpdatePrompt(data.value))
    }

    fn set_content(&self, content: ProblemContentBuilder) -> Callback<MouseEvent> {
        self.link
            .callback(move |_| CreateMsg::SetContent(content.clone()))
    }

    fn add_solution(&self, idx: usize) -> Callback<MouseEvent> {
        self.link.callback(move |_| CreateMsg::AddSolution(idx))
    }

    fn update_choice(&self) -> Callback<InputData> {
        self.link
            .callback(move |data: InputData| CreateMsg::UpdateChoice(data.value))
    }

    fn add_choice(&self) -> Callback<MouseEvent> {
        self.link.callback(|_| CreateMsg::AddChoice)
    }

    fn remove_choice(&self, idx: usize) -> Callback<MouseEvent> {
        self.link.callback(move |_| CreateMsg::RemoveChoice(idx))
    }

    fn finish_problem(&self) -> Callback<MouseEvent> {
        self.link.callback(|_| CreateMsg::Finish)
    }

    fn send_creation_request(&mut self, new_problem: NewProblem) -> FetchTask {
        let callback = self.link.callback(
            move |response: Response<Json<Result<i32, anyhow::Error>>>| {
                let (meta, Json(new_id)) = response.into_parts();
                if meta.status.is_success() {
                    match new_id {
                        Ok(new_id) => CreateMsg::CreationSuccess(new_id),
                        Err(error) => CreateMsg::CreationFailure(format!("{}", error)),
                    }
                } else {
                    CreateMsg::CreationFailure(format!("{}", meta.status))
                }
            },
        );
        let request = Request::post(format!("{}/problems/", API_URL))
            .header("Content-Type", "application/json")
            .body(Json(&new_problem))
            .unwrap();
        self.fetch_service.fetch(request, callback).unwrap()
    }
}

impl Component for CreateComponent {
    type Message = CreateMsg;
    type Properties = CreateProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|_| CreateMsg::NoOp);
        let router = RouteAgent::bridge(callback);

        Self {
            link,
            props,
            router,
            fetch_service: FetchService::new(),
            ft: None,
            builder: ProblemBuilder::default(),
            content_input_buffer: String::new(),
            new_tag_text: String::new(),
            error_message: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            CreateMsg::NoOp => false,
            CreateMsg::SetTopic(topic) => {
                self.builder.topic = Some(topic);
                true
            }
            CreateMsg::SetContent(content) => {
                self.builder.content = content;
                true
            }
            CreateMsg::RemoveTag(idx) => {
                self.builder.tags.remove(idx);
                true
            }
            CreateMsg::UpdateTagText(new_text) => {
                self.new_tag_text = new_text;
                false
            }
            CreateMsg::CreateTag => {
                if !self.new_tag_text.is_empty() {
                    self.builder.tags.push(self.new_tag_text.clone());
                }
                true
            }
            CreateMsg::UpdatePrompt(new_prompt) => {
                self.builder.prompt = new_prompt;
                false
            }
            CreateMsg::AddSolution(idx) => {
                match &mut self.builder.content {
                    ProblemContentBuilder::MultipleChoice { options, solution } => {
                        *solution = Some(idx)
                    }
                };
                match &mut self.builder.content {
                    ProblemContentBuilder::MultipleChoice { options, solution } => {
                        info!("{:?}", solution);
                    }
                };
                true
            }
            CreateMsg::UpdateChoice(choice_text) => {
                self.content_input_buffer = choice_text;
                false
            }
            CreateMsg::AddChoice => {
                match &mut self.builder.content {
                    ProblemContentBuilder::MultipleChoice { options, solution } => {
                        if !self.content_input_buffer.is_empty() {
                            options.push(self.content_input_buffer.clone());
                        }
                    }
                }
                true
            }
            CreateMsg::RemoveChoice(idx) => {
                match &mut self.builder.content {
                    ProblemContentBuilder::MultipleChoice { options, solution } => {
                        if idx >= options.len() {
                            return false;
                        }
                        options.remove(idx);
                        if let Some(s) = solution {
                            if *s > idx {
                                *s -= 1;
                            } else if *s == idx {
                                *solution = None
                            }
                        }
                    }
                }
                true
            }
            CreateMsg::Finish => {
                if let Some(topic) = &self.builder.topic {
                    if self.builder.prompt.is_empty() {
                        self.error_message = "Please enter a prompt".into();
                        return true;
                    }
                    match &self.builder.content {
                        ProblemContentBuilder::MultipleChoice { options, solution } => {
                            if options.len() <= 1 {
                                self.error_message =
                                    "Please provide at least 2 answer options".into();
                                return true;
                            }
                            if let Some(solution) = *solution {
                                let req = NewProblem {
                                    owner_id: self.props.user_id,
                                    topic: topic.clone(),
                                    tags: self.builder.tags.clone(),
                                    prompt: self.builder.prompt.clone(),
                                    content: ProblemContent::MultipleChoice {
                                        options: options.clone(),
                                        solution,
                                    },
                                    explanation: "".to_string() // TODO
                                };
                                self.error_message = "".into();
                                self.ft = Some(self.send_creation_request(req));
                            } else {
                                self.error_message = "Please select a solution by clicking on one of the response options".into();
                            }
                        }
                    }
                } else {
                    self.error_message = "You must specify a topic for your problem".into();
                }
                true
            }
            CreateMsg::CreationFailure(error_message) => {
                self.error_message = error_message;
                true
            }
            CreateMsg::CreationSuccess(_id) => {
                self.router
                    .send(RouteRequest::ChangeRoute(Route::from("/feed".to_string())));
                true
            }
        }
    }

    fn view(&self) -> Html {
        let topics = vec![Topic::Math, Topic::Trivia, Topic::Logic];
        let problem_types = vec![ProblemType::MultipleChoice, ProblemType::FreeResponse, ProblemType::Checklist];
        html! {
            <div class="createproblemwrapper">
                <div class="createproblem">
                    <div class="title">
                        { "Problem Creation Studio" }
                    </div>
                    <div class="errorbox">
                        { &self.error_message }
                    </div>
                    <div class="selector">
                        <div class="prompt">
                            { "Step 1: Pick a general topic" }
                        </div>
                        <div class="topicoptions">
                            {
                                for topics.iter().enumerate().map(|(i, topic)| {
                                    html! {
                                        <div
                                            class={
                                                let mut classes = "topic".to_string();
                                                if self.builder.topic == Some(topic.clone()) {
                                                    classes.push_str(" selected")
                                                }
                                                if i == 0 {
                                                    classes.push_str(" left")
                                                } else if i + 1 == topics.len() {
                                                    classes.push_str(" right")
                                                } else {
                                                    classes.push_str(" center")
                                                }
                                                classes
                                            }
                                            onclick=&self.set_topic(topic.clone())>
                                            {
                                                serde_json::to_string(&topic).unwrap().replace(r#"""#, "")
                                            }
                                        </div>
                                    }
                                })
                            }
                        </div>
                    </div>
                    <div class="selector">
                        <div class="prompt">
                            { "Step 2: Add any related tags" }
                        </div>
                        <div class="tags">
                            {
                                for self.builder.tags.iter().enumerate().map(|(i, tag)| html! {
                                    <div class="tag" onclick=&self.remove_tag(i)>{tag}</div>
                                })
                            }
                        </div>
                        <div class="addtag">
                            <input class="taginput" type="text" oninput=&self.update_tag_text() />
                            <button class="addtag" onclick=&self.create_tag()>{ "Add tag" }</button>
                        </div>
                    </div>
                    <div class="selector">
                        <div class="prompt">
                            { "Step 3: Write a prompt" }
                        </div>
                        <textarea class="promptinput" oninput=&self.update_prompt() />
                    </div>
                    <div class="selector">
                        <div class="prompt">
                            { "Step 4: Select an answer type" }
                        </div>
                        <div class="problemtypeoptions">
                            {
                                for problem_types.iter().enumerate().map(|(i, p_type)| {
                                    html! {
                                        <div
                                            class={
                                                let mut classes = "type".to_string();
                                                if self.builder.content.get_type() == *p_type {
                                                    classes.push_str(" selected");
                                                }
                                                if i == 0 {
                                                    classes.push_str(" left");
                                                } else if i + 1 == problem_types.len() {
                                                    classes.push_str(" right");
                                                } else {
                                                    classes.push_str(" center");
                                                }
                                                classes
                                            }
                                            onclick={
                                                if i == 0 {
                                                    self.set_content(ProblemContentBuilder::default_from_type(&p_type))
                                                } else {
                                                    self.link.callback(|_| CreateMsg::NoOp)
                                                }
                                            }
                                            >
                                            {
                                                serde_json::to_string(&p_type).unwrap().replace(r#"""#, "")
                                            }
                                        </div>
                                    }
                                })
                            }
                        </div>
                    </div>
                    <div class="selector">
                        {
                            match &self.builder.content {
                                ProblemContentBuilder::MultipleChoice { options, solution } => html!{
                                    <div class="multiplechoice">
                                        <div class="prompt">
                                            { "Step 5: Add choices and click the correct one" }
                                        </div>
                                        <div class="addchoice">
                                            <input type="text" oninput=&self.update_choice() />
                                            <button class="add" onclick=&self.add_choice()>{ "Add choice" }</button>
                                        </div>
                                        <div class="multiplechoicedisplay">
                                            {
                                                for options.iter().enumerate().map(|(i, option)| html! {
                                                    <div class="optionwrapper">
                                                        <span class="optionmarker" onclick=&self.remove_choice(i)>{ "X" }</span>
                                                        <div onclick=&self.add_solution(i) class={
                                                            if Some(i) == *solution {
                                                                "option selected"
                                                            } else {
                                                                "option"
                                                            }
                                                        }>{option}</div>
                                                    </div>
                                                })
                                            }
                                        </div>
                                    </div>
                                },
                                _ => unimplemented!("Not all problem types can be created rn")
                            }
                        }
                    </div>
                    <button class="submitproblem" onclick=&self.finish_problem()>
                        { "I'm all done!" }
                    </button>
                </div>
            </div>
        }
    }
}
