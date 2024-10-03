use crate::app::{App, InputField, InputMode};
use crate::date::{Date, DATE_FORMAT};
use crate::task::Task;
use anyhow::{Context, Result};

pub trait Command {
    fn execute(&mut self, app: &mut App)-> Result<()>;
}

pub struct EnterEditModeCommand;

impl Command for EnterEditModeCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        app.input_mode = InputMode::Editing;
        app.input_field = InputField::Title;
        Ok(())
    }
}

/// Add a new task
pub struct AddTaskCommand;

impl Command for AddTaskCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        let mut t = Task::new();
        t.title = app.input_title.drain(..).collect();
        t.description = app.input_description.drain(..).collect();
        if !app.input_date.is_empty() {
            t.date = Date::try_from(app.input_date.drain(..).collect::<String>()).context("Invalid date")?;
        }
        app.tasks_service.add_new_task(&t);
        app.refresh_task_list();
        Ok(())
    }
}

/// Toggle completed for selected task status
pub struct ToggleTaskStatusCommand;

impl Command for ToggleTaskStatusCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            let item = &mut app.task_list.items[index];
            item.completed = match item.completed {
                true => false,
                false => true,
            };
            let _ = app
                .tasks_service
                .toggle_task_status(item.id, item.completed);
        };
        Ok(())
    }
}

/// Switch between priorities
pub struct ToggleItemPriorityCommand;

impl Command for ToggleItemPriorityCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            let item = &mut app.task_list.items[index];
            item.priority = item.priority.next();
            app.tasks_service.change_priority(item.id, &item.priority);
        }
        Ok(())
    }
}

/// Start editing a task, move cursor to Title input field
/// and set InputMode equal to InputMode::EditingExisting
pub struct StartEditingExistingTaskCommand;

impl Command for StartEditingExistingTaskCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            app.input_title = app.task_list.items[index].title.clone();
            app.input_description = app.task_list.items[index].description.clone();
            app.input_date = app.task_list.items[index].date.clone().0.map(
                |d| d.format(DATE_FORMAT).to_string())
                .unwrap_or(String::new());
            app.input_mode = InputMode::EditingExisting;
            app.input_field = InputField::Title;
        }
        Ok(())
    }
}

/// Finish editing an existing task, set InputMode back to Normal
pub struct FinishEditingExistingTaskCommand;

impl Command for FinishEditingExistingTaskCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            app.task_list.items[index].title = app.input_title.drain(..).collect();
            app.task_list.items[index].description = app.input_description.drain(..).collect();
            if !app.input_date.is_empty() {
                app.task_list.items[index].date = Date::try_from(app.input_date.drain(..).collect::<String>())?;
            } else {
                app.task_list.items[index].date = Date(None)
            }
            app.tasks_service.update_task(&app.task_list.items[index])
        }
        app.input_mode = InputMode::Normal;
        Ok(())
    }
}

pub struct DeleteTaskCommand;

impl Command for DeleteTaskCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        if let Some(index) = app.task_list.state.selected() {
            app.tasks_service.delete_task(app.task_list.items[index].id);
            app.task_list.items.remove(index);
        }
        Ok(())
    }
}

/// Stop adding or editing the current task, clear the input fields and
/// set InputMode back to Normal
pub struct StopEditingCommand;

impl Command for StopEditingCommand {
    fn execute(&mut self, app: &mut App) -> Result<()> {
        app.input_mode = InputMode::Normal;
        app.input_title.clear();
        app.input_description.clear();
        app.input_date.clear();
        Ok(())
    }
}
