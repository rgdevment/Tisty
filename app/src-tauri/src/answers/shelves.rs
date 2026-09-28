use std::sync::Mutex;

use tisty_core::{List, Op, State};

use crate::{Answer, Refusal, Session, folder_open, held, stop, tray};

#[tauri::command]
pub fn list_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Answer<List> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    let mut session = held(&session);
    if !session.state.list_called(&name).is_empty() {
        return Err(Refusal::about("manyLists", name));
    }

    let id = ulid::Ulid::generate();
    let order = session.state.next_list_order();
    session.commit(Op::ListAdd {
        id,
        d: tisty_core::event::ListAdd {
            name,
            order,
            color: None,
        },
    })?;
    let painted = color.filter(|key| tisty_core::model::hue::kept(key).is_some());
    let drawn = icon.filter(|key| tisty_core::model::icon::known(key));
    if drawn.is_some() || painted.is_some() {
        session.commit(Op::ListLook {
            id,
            d: tisty_core::event::Look {
                icon: Some(drawn),
                color: Some(painted),
            },
        })?;
    }
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
pub fn list_look(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    icon: Option<String>,
    color: Option<String>,
) -> Answer<List> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let kept = match icon {
        Some(key) => Some(
            tisty_core::model::icon::kept(&key).ok_or_else(|| Refusal::about("noSuchIcon", key))?,
        ),
        None => None,
    };
    let painted = match color {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchColour", key))?,
        ),
        None => None,
    };

    let mut session = held(&session);
    session.commit(Op::ListLook {
        id,
        d: tisty_core::event::Look {
            icon: Some(kept),
            color: Some(painted),
        },
    })?;
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
pub fn list_rename(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<List> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let name = tisty_core::text::plainly(&name);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }

    let mut session = held(&session);
    if !session.state.lists.contains_key(&id) {
        return Err(Refusal::of("notAListId"));
    }
    if session
        .state
        .list_called(&name)
        .iter()
        .any(|one| one.id != id)
    {
        return Err(Refusal::about("manyLists", name));
    }

    session.commit(Op::ListRename {
        id,
        d: tisty_core::event::Name { name },
    })?;
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
pub fn list_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let mut session = held(&session);
    if !session.state.lists.contains_key(&id) {
        return Err(Refusal::of("notAListId"));
    }
    session.commit(Op::ListDelete { id })?;
    Ok(())
}

#[tauri::command]
pub fn folder_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    parent: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Answer<String> {
    let name = named_folder(&name)?;
    let parent = parent
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let painted = match color {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchColour", key))?,
        ),
        None => None,
    };

    let mut session = held(&session);
    if let Some(at) = parent {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        folder_open(&session.state, at, true)?;
        if session.state.depth(Some(at)) >= tisty_core::model::DEEPEST {
            return Err(Refusal::of("tooDeep"));
        }
    }
    let order = tisty_core::order::last_of(
        session
            .state
            .under(parent)
            .iter()
            .map(|one| one.order.as_str()),
    );
    let id = ulid::Ulid::generate();
    session.commit(Op::FolderAdd {
        id,
        d: tisty_core::event::FolderAdd {
            name,
            order,
            parent,
            icon: icon.filter(|key| tisty_core::model::icon::known(key)),
            color: painted,
        },
    })?;
    Ok(id.to_string())
}

#[tauri::command]
pub fn folder_rename(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let name = named_folder(&name)?;
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(&session.state, id, false)?;
    session.commit(Op::FolderRename {
        id,
        d: tisty_core::event::Name { name },
    })?;
    Ok(())
}

#[tauri::command]
pub fn folder_look(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    icon: Option<String>,
    color: Option<String>,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let kept = match icon {
        Some(key) => Some(
            tisty_core::model::icon::kept(&key).ok_or_else(|| Refusal::about("noSuchIcon", key))?,
        ),
        None => None,
    };
    let painted = match color {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchColour", key))?,
        ),
        None => None,
    };
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(&session.state, id, false)?;
    session.commit(Op::FolderLook {
        id,
        d: tisty_core::event::Look {
            icon: Some(kept),
            color: Some(painted),
        },
    })?;
    Ok(())
}

#[tauri::command]
pub fn folder_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(&session.state, id, false)?;
    session.commit(Op::FolderDelete { id })?;
    Ok(())
}

#[tauri::command]
pub fn folder_file(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    parent: Option<String>,
    before: Option<String>,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let parent = parent
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let before: Option<tisty_core::model::FolderId> = before
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;

    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(&session.state, id, false)?;
    if let Some(at) = parent {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        folder_open(&session.state, at, true)?;
        if session.state.would_swallow(id, at) {
            return Err(Refusal::of("intoItself"));
        }
        if session.state.depth(Some(at)) + session.state.tall_under(id) > tisty_core::model::DEEPEST
        {
            return Err(Refusal::of("tooDeep"));
        }
    }
    if before.is_some_and(|at| at == id) {
        return Err(Refusal::of("intoItself"));
    }
    let ops = beside_folders(&session.state, id, parent, before);
    session.commit_all(ops)?;
    Ok(())
}

#[tauri::command]
pub fn folder_away(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    away: bool,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let mut session = held(&session);
    let Some(folder) = session.state.folders.get(&id) else {
        return Err(Refusal::of("noSuchFolder"));
    };
    // A folder inside one the archive already holds has no say of its own, either way.
    if folder
        .parent
        .is_some_and(|up| session.state.folder_away(up))
    {
        return Err(Refusal::of("folderAway"));
    }
    if folder.archived == away {
        return Ok(());
    }
    session.commit(if away {
        Op::FolderArchive { id }
    } else {
        Op::FolderUnarchive { id }
    })?;
    Ok(())
}

#[tauri::command]
pub fn sow(app: tauri::AppHandle, priority: Option<String>) {
    tray::sow(&app, priority);
}

#[tauri::command]
pub fn sow_lists(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    held(&session).sow_if_due();
    Ok(())
}

pub fn named_folder(said: &str) -> Answer<String> {
    let name = tisty_core::text::plainly(said);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    // A slash reads as a path everywhere else, and then one name stands for two folders.
    if name.contains('/') {
        return Err(Refusal::of("folderNameSlash"));
    }
    if name.chars().count() > tisty_core::model::FOLDER_NAME_AT_MOST {
        return Err(Refusal::of("folderNameTooLong"));
    }
    Ok(name)
}

pub fn beside_folders(
    state: &State,
    id: tisty_core::model::FolderId,
    parent: Option<tisty_core::model::FolderId>,
    before: Option<tisty_core::model::FolderId>,
) -> Vec<Op> {
    let sitting: Vec<&tisty_core::model::Folder> = state
        .under(parent)
        .into_iter()
        .filter(|one| one.id != id)
        .collect();
    let keys: Vec<&str> = sitting.iter().map(|one| one.order.as_str()).collect();
    let (mine, fresh) = tisty_core::order::dealt(
        &keys,
        stop(before.map(|at| sitting.iter().position(|one| one.id == at))),
    );
    let mut ops: Vec<Op> = sitting
        .iter()
        .zip(fresh)
        .filter_map(|(one, order)| {
            order.map(|order| Op::FolderMove {
                id: one.id,
                d: tisty_core::event::Filed {
                    folder: None,
                    page_of: None,
                    order: Some(order),
                },
            })
        })
        .collect();
    ops.push(Op::FolderMove {
        id,
        d: tisty_core::event::Filed {
            folder: Some(parent),
            page_of: None,
            order: Some(mine),
        },
    });
    ops
}
