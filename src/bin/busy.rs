extern crate chrono;
extern crate clap;
extern crate clap_complete;
extern crate colored;
extern crate serde;
extern crate serde_json;
extern crate uuid;

mod commands;
mod view;

use std::{
  cell::RefCell,
  collections::HashSet,
  io::{Read, Seek, Write},
  rc::Rc,
};

use busy::{
  duration::{get_midnight_datetime, get_period_since_now, get_week_start_datetime, Period},
  Busy,
};

use busy::task::Task;
use busy::task::TaskView;
use busy::time::parse_datetime;
use busy::traits::Indexable;
use clap::{CommandFactory, Parser};
use colored::Colorize;
use view::viewer::Viewer;

#[derive(clap::Parser)]
struct BusyCli {
  #[clap(subcommand)]
  command: commands::Commands,
}

fn main() {
  env_logger::init();

  let cli = BusyCli::parse();

  let busy = Rc::new(RefCell::new(Busy::new()));
  let viewer = Viewer::new(Rc::clone(&busy));

  match &cli.command {
    commands::Commands::Start(params) => {
      let mut start_time = None;
      if let Some(start_time_str) = params.start_time.as_ref() {
        let parsed_start_time = parse_datetime(start_time_str);
        if parsed_start_time.is_err() {
          eprintln!(
            "Can't parse start-time parameter: {start_time_str}, err: {:?}",
            parsed_start_time.err()
          );
          return;
        }
        start_time = Some(parsed_start_time.unwrap());
      }

      let started_task_result = {
        busy.borrow_mut().start(
          &params.project_name,
          &params.task_title,
          params.tags.clone(),
          start_time,
        )
      };
      match started_task_result {
        Ok(task) => {
          println!("Task started:");
          viewer.log_task(&task, true);
        }
        Err(err) => eprintln!("start task err: {err}"),
      };
    }

    commands::Commands::Stop => {
      let stopped_task_result = { busy.borrow_mut().stop() };
      match stopped_task_result {
        Ok(task) => {
          println!("Task stopped:");
          viewer.log_task(&task, true);
        }
        Err(err) => eprintln!("couldn't stop: {err}"),
      };
    }

    commands::Commands::Pause => {
      let paused_task_result = { busy.borrow_mut().pause() };
      match paused_task_result {
        Ok(task) => {
          println!("Task paused:");
          viewer.log_task(&task, true);
        }
        Err(err) => eprintln!("couldn't pause: {err}"),
      };
    }

    commands::Commands::Resume => {
      let unpaused_task_result = { busy.borrow_mut().resume() };
      match unpaused_task_result {
        Ok(task) => {
          println!("Task resumed:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("couldn't resume: {err}"),
      };
    }

    commands::Commands::Continue(params) => {
      let task_id = restore_id_by_short_id(Rc::clone(&busy), &params.short_task_id);
      if task_id.is_err() {
        eprintln!(
          "Continue parse short id into uuid error: {:?}",
          task_id.err()
        );
        return;
      }

      let task = busy.borrow_mut().continue_task(task_id.unwrap());
      if task.is_err() {
        eprintln!("Continue task error: {:?}", task.err());
        return;
      }

      println!("Continue task:");
      viewer.log_task(task.as_ref().unwrap(), true);
    }

    commands::Commands::Tags => {
      clear_screen();
      println!("{}", "Tags: ".bright_cyan());
      viewer.print_tags();
    }

    commands::Commands::Projects => {
      clear_screen();
      println!("{}", "Projects: ".bright_cyan());
      viewer.print_projects();
    }

    commands::Commands::Add(params) => {
      let start_time = parse_datetime(&params.start_time);
      let finish_time = parse_datetime(&params.finish_time);

      if start_time.is_err() || finish_time.is_err() {
        eprintln!(
          "failed to parse start or finish time: {:?} {:?}",
          start_time.err(),
          finish_time.err()
        );
        return;
      }

      let started_task_result = {
        busy.borrow_mut().add(
          &params.project_name,
          &params.task_title,
          params.tags.clone(),
          start_time.unwrap(),
          finish_time.unwrap(),
        )
      };

      match started_task_result {
        Ok(task) => {
          println!("Task added:");
          viewer.log_task(&task, true);
        }
        Err(err) => println!("add task err: {err}"),
      };
    }

    commands::Commands::Remove(params) => {
      let task_id = restore_id_by_short_id(Rc::clone(&busy), &params.short_task_id);
      if task_id.is_err() {
        eprintln!("Parse short id into uuid error: {:?}", task_id.err());
        return;
      }

      let task: Task;
      {
        let mut p = busy.borrow_mut();
        task = p.task_by_id(task_id.unwrap()).unwrap();
        p.remove_task(task.id()).unwrap();
      };
      println!("Removed task:");
      viewer.log_task(&task, true);
    }

    commands::Commands::Edit(params) => {
      if params.all_tags {
        edit(
          Rc::clone(&busy),
          &viewer,
          EditDataType::AllTags,
          uuid::Uuid::new_v4(),
        );
        return;
      }

      if params.all {
        edit(
          Rc::clone(&busy),
          &viewer,
          EditDataType::AllTasks,
          uuid::Uuid::new_v4(),
        );
        return;
      }

      let extract_ids_and_edit = |short_ids: Option<&Vec<String>>, edit_type: EditDataType| {
        if short_ids.is_none() {
          return;
        }

        let ids: Vec<uuid::Uuid> = short_ids
          .unwrap()
          .iter()
          .map(|short_id| restore_id_by_short_id(Rc::clone(&busy), short_id).unwrap())
          .collect();

        for id in ids {
          edit(Rc::clone(&busy), &viewer, edit_type, id);
        }
      };

      extract_ids_and_edit(params.task_id.as_ref(), EditDataType::Task);
      extract_ids_and_edit(params.project_id.as_ref(), EditDataType::Project);
      extract_ids_and_edit(params.tag_id.as_ref(), EditDataType::Tag);

      println!("\nEdit completed");
    }

    commands::Commands::Status => {
      match busy.borrow().active_task() {
        Some(task) => {
          println!("Your active task:");
          viewer.log_task(&task, true);
        }
        None => {
          eprintln!("There are no active tasks");
        }
      };
    }

    commands::Commands::Log(params) => {
      show_tasks(
        &params.log_params,
        Rc::clone(&busy),
        &viewer,
        get_period(params.days, params.today),
      );
    }

    commands::Commands::Today(params) => {
      show_tasks(
        &params.log_params,
        Rc::clone(&busy),
        &viewer,
        Period::new_to_now(get_midnight_datetime()),
      );
    }

    commands::Commands::Stat(params) => {
      if !params.log_params.dont_clear {
        clear_screen();
      }

      let project_ids = projects_to_ids_set(Rc::clone(&busy), &params.log_params.project);
      let found_tags = busy.borrow().find_tag_by_names(&params.log_params.tag);

      viewer.show_stat(
        get_period(params.days, params.today),
        project_ids,
        &found_tags,
        params.with_tags,
      );
    }

    commands::Commands::Sync(params) => {
      if params.push_force {
        println!("Start sync push force…");
        match busy.borrow_mut().push_force() {
          Ok(_) => println!("Sync push force success!"),
          Err(err) => eprintln!("Sync push force failed, output:\n{err}"),
        };
      } else if params.pull_force {
        println!("Start sync pull force…");
        match busy.borrow_mut().pull_force() {
          Ok(_) => println!("Sync pull force success!"),
          Err(err) => eprintln!("Sync pull force failed, output:\n{err}"),
        };
      } else {
        println!("Start syncing…");
        let sync_result = busy.borrow_mut().sync();
        match sync_result {
          Ok(_) => {
            println!("Syncing finished");
          }
          Err(err) => {
            eprintln!("Sync failed, err output:\n{err}");
            eprintln!("You can try to use `busy sync --push-force` or `busy sync --pull-force`");
          }
        };
      }
    }

    commands::Commands::Complete(completions) => {
      completions.complete(&mut BusyCli::command());
    }
  };
}

fn show_tasks(
  params: &commands::LogCommonParams,
  busy: Rc<RefCell<Busy>>,
  viewer: &Viewer,
  period: Period,
) {
  if !params.dont_clear {
    clear_screen();
  }

  let project_ids = projects_to_ids_set(Rc::clone(&busy), params.project.as_slice());
  let found_tags = busy.borrow().find_tag_by_names(&params.tag);

  viewer.log_tasks_list(period, project_ids, &found_tags, params.full);
}

#[derive(Debug, Clone, Copy)]
enum EditDataType {
  Task,
  Project,
  Tag,
  AllTags,
  AllTasks,
}

fn get_editor() -> String {
  std::env::var("EDITOR").unwrap_or(std::env::var("VISUAL").unwrap_or("nvim".to_string()))
}

fn run_edit_and_get_result<T: serde::ser::Serialize + serde::de::DeserializeOwned>(
  item: &T,
  tmp_file: &mut tempfile::NamedTempFile,
  editor: &str,
) -> T {
  let item_str = serde_json::to_string_pretty(item).unwrap();
  tmp_file.write_all(item_str.as_bytes()).unwrap();

  subprocess::Exec::cmd(editor)
    .arg(tmp_file.path())
    .join()
    .expect("edit cmd doesn't work");

  let mut buf = String::new();
  tmp_file.seek(std::io::SeekFrom::Start(0)).unwrap();
  tmp_file.read_to_string(&mut buf).unwrap();

  log::debug!("edit result: {buf}");

  return serde_json::from_str(&buf).expect("can't decode item back, please try again");
}

fn edit(busy: Rc<RefCell<Busy>>, viewer: &Viewer, edit_data_type: EditDataType, id: uuid::Uuid) {
  let editor = get_editor();
  let mut tmp_file = tempfile::Builder::new()
    .prefix("busy_")
    .suffix(".json")
    .tempfile()
    .unwrap();

  log::debug!("edit {edit_data_type:?} id: {id} tmp_file_path: {tmp_file:?} editor: {editor}");

  match edit_data_type {
    EditDataType::Task => {
      let task = busy.borrow().task_by_id(id).unwrap();
      let mut all_tags = busy.borrow().tags();
      let task_view = TaskView::from_task(&task, &all_tags);

      let updated_task_view = run_edit_and_get_result(&task_view, &mut tmp_file, &editor);

      let new_tags = updated_task_view.resolve_new_tags(&all_tags);
      busy.borrow_mut().upsert_tags(new_tags);
      all_tags = busy.borrow().tags();

      let updated_task = updated_task_view.to_task(&all_tags);
      viewer.log_task(&updated_task, true);
      busy.borrow_mut().replace_task(&updated_task).unwrap();
    }

    EditDataType::Project => {
      let project = busy.borrow().project_by_id(id).unwrap();
      let updated_project = run_edit_and_get_result(&project, &mut tmp_file, &editor);

      println!("{}", "Updated project: ".bright_yellow());
      viewer.print_project(&updated_project);
      busy.borrow_mut().replace_project(&updated_project).unwrap();
    }

    EditDataType::Tag => {
      let tag = busy.borrow().tag_by_id(id).unwrap();
      let updated_tag = run_edit_and_get_result(&tag, &mut tmp_file, &editor);

      println!("{}", "Updated tag: ".bright_yellow());
      viewer.print_tag(&updated_tag);
      busy.borrow_mut().replace_tag(&updated_tag).unwrap();
    }

    EditDataType::AllTags => {
      let edited_data =
        run_edit_and_get_result(&busy.borrow().all_tags(), &mut tmp_file, editor.as_str());
      busy.borrow_mut().replace_tags(edited_data);
      println!("Edit finished, tags were saved");
    }

    EditDataType::AllTasks => {
      let edited_data =
        run_edit_and_get_result(&busy.borrow().all_tasks(), &mut tmp_file, editor.as_str());
      busy.borrow_mut().replace_tasks(edited_data);
      println!("Edit finished, tasks were saved");
    }
  };
}

fn clear_screen() {
  if log::log_enabled!(log::Level::Debug) {
    return;
  }
  subprocess::Exec::cmd("clear")
    .join()
    .expect("clean cmd doesn't work");
}

fn projects_to_ids_set(
  busy: Rc<RefCell<Busy>>,
  project_names: &[String],
) -> Option<HashSet<uuid::Uuid>> {
  let mut project_ids = HashSet::new();
  for project_name in project_names.iter() {
    let project = busy.borrow().project_by_name(project_name);
    if project.is_some() {
      project_ids.insert(project.unwrap().id().clone());
    }
  }
  if project_ids.is_empty() {
    return None;
  }
  return Some(project_ids);
}

fn get_period(days: Option<i64>, today: bool) -> Period {
  if today {
    return Period::new_to_now(get_midnight_datetime());
  }

  match days {
    Some(n) => Period::new_to_now(get_period_since_now(n)),
    None => Period::new_to_now(get_week_start_datetime()),
  }
}

fn restore_id_by_short_id(busy: Rc<RefCell<Busy>>, short_id: &str) -> anyhow::Result<uuid::Uuid> {
  match busy.borrow().resolve_id(short_id) {
    Some(id) => Ok(id.clone()),
    None => anyhow::bail!("id by short name: {short_id} not found"),
  }
}
