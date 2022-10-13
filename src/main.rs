use yew::prelude::*;

enum Msg {
    AddOne,
}

struct Model {
    value: i64,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { value: 0 }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::AddOne => {
                self.value += 1;
                // the value has changed so we need to
                // re-render for it to appear on the page
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // This gives us a component's "`Scope`" which allows us to send messages, etc to the component.
        let link = ctx.link();
        html! {
            <div>
                <button onclick={link.callback(|_| Msg::AddOne)}>{ "+1" }</button>
                <p>{ self.value }</p>
            </div>
        }
    }
}

use glob::glob_with;
use glob::MatchOptions;

#[derive(Debug)]
struct Post {
    relative_path: String,
    tags: Option<Vec<String>>,
    title: String,
    description: Option<String>,
    draft: Option<bool>,
    content: String,
    date: Option<DateTime<Utc>>,
}

fn get_posts(dir: &str) -> std::vec::Vec<Post> {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let markdown_glob = "content/posts/*.md";
    glob_with(format!("{}/{}", dir, markdown_glob).as_str(), options)
        .unwrap()
        .filter_map(|p| p.ok())
        .filter_map(|pb| get_post_from_path(&pb))
        .collect::<Vec<_>>()
}

use chrono::{DateTime, Utc};
use markdown_parser;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct PostConfig {
    title: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    draft: Option<bool>,
    date: Option<DateTime<Utc>>,
}

fn get_post_from_path(path: &std::path::PathBuf) -> Option<Post> {
    // reading the path and parse
    //let file_result = std::fs::read_to_string(path);
    // using markdown parser to read
    let markdown_result = markdown_parser::read_file(path);
    // parsing different tags
    match markdown_result {
        Ok(content) => {
            match content.adapt::<markdown_parser::TomlAdapter, markdown_parser::BasicObject>() {
                Ok(md) => {
                    let content = md.content().clone();
                    let front_matter = md.front_matter();
                    let post_config: PostConfig = toml::from_str(front_matter).unwrap();
                    Some(Post {
                        relative_path: String::from(path.to_str().unwrap()),
                        tags: post_config.tags,
                        title: post_config.title,
                        description: post_config.description,
                        draft: post_config.draft,
                        content: content,
                        date: post_config.date,
                    })
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

fn main() {
    let dir = "/Users/dongbinli/sites/orchardlabdev-site";
    let posts = get_posts(dir);
    // parse all the results and convert them into Post array
    println!("{:?}", posts);
    //yew::start_app::<Model>();
}
