use glob::glob_with;
use glob::MatchOptions;

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    relative_path: String,
    tags: Option<Vec<String>>,
    title: String,
    description: Option<String>,
    draft: Option<bool>,
    content: String,
    date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatePostRequest {
    title: String,
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
        .filter_map(|pb| get_post_from_path(dir, &pb))
        .collect::<Vec<_>>()
}

use chrono::{DateTime, Utc};
use markdown_parser;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
struct PostConfig {
    title: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    draft: Option<bool>,
    date: Option<DateTime<Utc>>,
}

fn get_post_from_relative_path(dir: &str, relative_path: String) -> Option<Post> {
    let full_path = format!("{}/{}", dir, relative_path);
    let path_buf = std::path::PathBuf::from(full_path);
    get_post_from_path(dir, &path_buf)
}

fn get_post_from_path(dir: &str, path: &std::path::PathBuf) -> Option<Post> {
    // reading the path and parse
    //let file_result = std::fs::read_to_string(path);
    // using markdown parser to read
    let markdown_result = markdown_parser::read_file(path);

    // relative path
    let relative_path = path.strip_prefix(dir).unwrap();
    // parsing different tags
    match markdown_result {
        Ok(content) => {
            match content.adapt::<markdown_parser::TomlAdapter, markdown_parser::BasicObject>() {
                Ok(md) => {
                    let content = md.content().clone();
                    let front_matter = md.front_matter();
                    let post_config: PostConfig = toml::from_str(front_matter).unwrap();
                    Some(Post {
                        relative_path: String::from(relative_path.to_str().unwrap()),
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

// can not get post from path error
use std::{error::Error, fmt};

#[derive(Debug)]
struct GeneralError;

impl Error for GeneralError {}

impl fmt::Display for GeneralError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

// update post content
// Box<dyn std::error::Error> means any error?
fn update_post_content(
    dir: &str,
    new_content: String,
    relative_path: String,
) -> Result<Post, Box<dyn std::error::Error>> {
    // getting full path
    let path_string = format!("{}/{}", dir, relative_path);
    let path = std::path::Path::new(&path_string);
    // get the content
    let content = markdown_parser::read_file(path)?;
    let mut md = content.adapt::<markdown_parser::TomlAdapter, markdown_parser::BasicObject>()?;

    // modify the original content
    md.set_content(new_content);

    // set front matter
    // serialize the PostConfig and set here
    // we will have a separate function dealing with this
    // md.set_front_matter(String::from(""));

    match md.write_file(path) {
        Ok(_) => {
            let path_buf = path.to_path_buf();
            match get_post_from_path(dir, &path_buf) {
                Some(post) => Ok(post),
                None => Err(Box::new(GeneralError {})),
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn to_slug(str: String) -> String {
    slug::slugify(str)
}

use std::process::Command;

// create post
fn create_new_post(dir: &str, title: String) -> Option<Post> {
    // turn given title into slug
    let slug = to_slug(title);
    let relative_path = format!("content/posts/{}.md", slug);
    let result = Command::new("/opt/homebrew/bin/hugo")
        .current_dir(dir)
        .arg("new")
        .arg(relative_path.clone()) // passing clone in so it won't hold the ref / or borrowed
        .output();
    // trigger hugo command to create that
    match result {
        Ok(_) => {
            // read post back
            get_post_from_relative_path(dir, relative_path)
        }
        Err(_) => None,
    }
}

use warp::Filter;

//mod handlers {

use std::convert::Infallible;
use warp::http::StatusCode;

pub async fn list_projects() -> Result<impl warp::Reply, Infallible> {
    // Just return a JSON array of todos, applying the limit and offset.
    let dir = "/Users/dongbinli/sites/orchardlabdev-site";
    let posts = get_posts(dir);
    //let todos: Vec<Post> = posts.clone().into_iter().collect();
    Ok(warp::reply::json(&posts))
}

fn json_body() -> impl Filter<Extract = (Post,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_for_post_creation(
) -> impl Filter<Extract = (CreatePostRequest,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

async fn create_post(
    create_post_req: CreatePostRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("about to create post with info like: {:?}", create_post_req);

    let dir = "/Users/dongbinli/sites/orchardlabdev-site";
    let result = create_new_post(dir, create_post_req.title);
    Ok(warp::reply::json(&result))
}

pub fn posts_create() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("posts")
        .and(warp::post())
        .and(json_body_for_post_creation())
        .and_then(create_post)
}

async fn update_post(post: Post) -> Result<impl warp::Reply, warp::Rejection> {
    println!("about to update post like: {:?} ", post);
    let dir = "/Users/dongbinli/sites/orchardlabdev-site";

    match update_post_content(dir, post.content, post.relative_path) {
        Ok(updated_post) => Ok(warp::reply::json(&updated_post)),
        Err(_) => Ok(warp::reply::json(&String::from("somwthing wrong"))),
    }
}

/// PUT /todos/:id with JSON body
pub fn posts_update() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("posts")
        .and(warp::put())
        .and(json_body())
        .and_then(update_post)
}

//}

#[tokio::main]
async fn main() {
    // https://github.com/seanmonstar/warp/blob/master/examples/todos.rs
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let routes = warp::path!("posts")
        .and(warp::get())
        .and_then(list_projects)
        .or(posts_create())
        .or(posts_update());

    // routes = routes.or

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn test_main() {
    let dir = "/Users/dongbinli/sites/orchardlabdev-site";
    let posts = get_posts(dir);
    // parse all the results and convert them into Post array
    println!("{:?}", posts);

    // trying update for now
    let rr = update_post_content(
        dir,
        String::from("here is the updated content now..."),
        String::from("content/posts/just-test-rust-client.md"),
    );
    println!("=======> {:?}", rr);

    // test creating new post
    let post = create_new_post(dir, String::from("This is a brave new world!!"));
    println!("++++++++++ NEW Post: {:?}", post);
    //yew::start_app::<Model>();

    // start web server
}
