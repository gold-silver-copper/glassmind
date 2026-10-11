use glassmind_stories::{ActionModel, Reply, World};
use serde::Deserialize;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
#[derive(Deserialize)]
struct Request {
    #[serde(default)]
    story: Option<Vec<String>>,
    #[serde(default)]
    event: Option<String>,
    #[serde(default)]
    question: Option<String>,
    #[serde(default)]
    reset: bool,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut json = false;
    let mut model_path = None;
    let mut training_path = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--jsonl" => json = true,
            "--model" => {
                model_path = Some(args.next().ok_or("--model requires a file")?);
            }
            "--train" => {
                training_path = Some(args.next().ok_or("--train requires a file")?);
            }
            "--help" => {
                println!(
                    "glassmind-stories [--jsonl] [--model FILE]\nglassmind-stories --train FILE < training-stories.jsonl\nPlain input: one assertion/question per line; /reset clears memory.\nJSONL: {{story:[assertions], question:string}} or {{event:string}} or {{reset:true}}."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }
    if let Some(path) = training_path {
        let mut model = ActionModel::default();
        for line in io::stdin().lock().lines() {
            let request: Request = serde_json::from_str(&line?)?;
            let story = request.story.ok_or("training requires a complete story")?;
            model.learn(&story);
        }
        std::fs::write(path, serde_json::to_string_pretty(&model)?)?;
        eprintln!(
            "learned from {} stories; {} unsupported stories skipped",
            model.stories_seen, model.stories_skipped
        );
        return Ok(());
    }
    let model = Arc::new(match model_path {
        Some(path) => serde_json::from_str(&std::fs::read_to_string(path)?)?,
        None => ActionModel::default(),
    });
    let mut world = World::with_model(model.clone());
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in io::stdin().lock().lines() {
        let line = line?;
        let reply = if json {
            match serde_json::from_str::<Request>(&line) {
                Ok(r) => {
                    if r.reset || r.story.is_some() {
                        world = World::with_model(model.clone());
                    }
                    if let Some(story) = r.story {
                        for event in story {
                            world.observe(&event);
                        }
                    }
                    let observation = r.event.map(|event| world.observe(&event));
                    if let Some(q) = r.question {
                        world.ask(&q)
                    } else {
                        observation.unwrap_or(Reply {
                            answer: None,
                            reason: "reset".into(),
                        })
                    }
                }
                Err(err) => Reply {
                    answer: None,
                    reason: format!("invalid request: {err}"),
                },
            }
        } else if line.trim() == "/reset" {
            world = World::with_model(model.clone());
            Reply {
                answer: None,
                reason: "reset".into(),
            }
        } else if line.trim().ends_with('?') {
            world.ask(&line)
        } else {
            world.observe(&line)
        };
        if json {
            serde_json::to_writer(&mut out, &reply)?;
            writeln!(out)?;
        } else {
            writeln!(out, "{}", reply.answer.as_deref().unwrap_or(&reply.reason))?;
        }
        out.flush()?;
    }
    Ok(())
}
