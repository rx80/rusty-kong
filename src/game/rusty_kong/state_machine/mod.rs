// --------------------------------------------------------------------------
//
// Rusty Kong
// Copyright (C) 2018 Jeff Panici
// All rights reserved.
//
// This software source file is licensed according to the
// MIT License.  Refer to the LICENSE file distributed along
// with this source file to learn more.
//
// --------------------------------------------------------------------------

use std::cell::RefCell;
use std::marker::Sync;
use std::fmt::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum GameState {
    None = 0 as isize,
    Boot,
    Attract,
    LongIntroduction,
    HowHigh,
    GamePlay,
    PlayerDies,
    PlayerWins,
    KongRetreats,
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            &GameState::None                => write!(f, "state_nop"),
            &GameState::Boot                => write!(f, "boot"),
            &GameState::Attract             => write!(f, "attract"),
            &GameState::LongIntroduction    => write!(f, "long_intro"),
            &GameState::HowHigh             => write!(f, "how_high"),
            &GameState::GamePlay            => write!(f, "game_play"),
            &GameState::PlayerDies          => write!(f, "player_dies"),
            &GameState::PlayerWins          => write!(f, "player_wins"),
            &GameState::KongRetreats        => write!(f, "kong_retreats")
        }
    }
}

struct StateHandlers {
    enter: fn(),
    update: fn(),
    leave: fn(),
    first_update: RefCell<bool>
}

unsafe impl Sync for StateHandlers {
}

struct States {
    previous: GameState,
    current: GameState,
    next: GameState,
}

thread_local!(
    static STATE:RefCell<States> = RefCell::new(States {
        previous: GameState::None,
        current:  GameState::None,
        next:     GameState::None
    });
);

mod boot;
use self::boot::*;

mod attract;
use self::attract::*;

mod long_introduction;
use self::long_introduction::*;

mod how_high;
use self::how_high::*;

mod game_play;
use self::game_play::*;

mod player_dies;
use self::player_dies::*;

mod player_wins;
use self::player_wins::*;

mod kong_retreats;
use self::kong_retreats::*;

mod state_nop;
use self::state_nop::*;

#[allow(dead_code)]
lazy_static! {
    static ref STATE_HANDLERS:Vec<StateHandlers> = vec!(
        StateHandlers {
            enter: state_nop,
            update: state_nop,
            leave: state_nop,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: boot_enter,
            update: boot_update,
            leave: boot_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: attract_enter,
            update: attract_update,
            leave: attract_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: long_intro_enter,
            update: long_intro_update,
            leave: long_intro_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: how_high_enter,
            update: how_high_update,
            leave: how_high_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: game_play_enter,
            update: game_play_update,
            leave: game_play_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: player_dies_enter,
            update: player_dies_update,
            leave: player_dies_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: player_wins_enter,
            update: player_wins_update,
            leave: player_wins_leave,
            first_update: RefCell::new(true)
        },

        StateHandlers {
            enter: kong_retreats_enter,
            update: kong_retreats_update,
            leave: kong_retreats_leave,
            first_update: RefCell::new(true)
        },
    );
}

fn get_previous_state() -> GameState {
    STATE.with(|cell| cell.borrow().previous)
}

fn set_previous_state(state:GameState) {
    STATE.with(|cell| {cell.borrow_mut().previous = state;});
}

fn get_next_state() -> GameState {
    STATE.with(|cell| cell.borrow().next)
}

fn set_next_state(state:GameState) {
    STATE.with(|cell| {cell.borrow_mut().next = state;});
}

fn get_current_state() -> GameState {
    STATE.with(|cell| cell.borrow().current)
}

fn set_current_state(state:GameState) {
    STATE.with(|cell| {cell.borrow_mut().current = state;});
}

pub fn game_state_go(state:GameState) {
    STATE.with(|cell| {cell.borrow_mut().next = state;});
}

pub fn game_state_init() {
    game_state_go(GameState::Boot);
}

fn get_state_handlers<'a>(state: GameState) -> &'a StateHandlers {
    &STATE_HANDLERS[state as usize]
}

pub fn game_state_update() {
    if get_next_state() != GameState::None {
        set_previous_state(get_current_state());
        debug!("transition from: {}.", get_previous_state());
        let previous_handlers = get_state_handlers(get_previous_state());
        debug!("calling {}_leave().", get_previous_state());
        (previous_handlers.leave)();
        let mut first_update = previous_handlers.first_update.borrow_mut();
        *first_update = true;

        set_current_state(get_next_state());
        debug!("transition to: {}.", get_current_state());
        set_next_state(GameState::None);

        let current_handlers = get_state_handlers(get_current_state());
        debug!("calling {}_enter.", get_current_state());
        (current_handlers.enter)();
    } else {
        let handlers = get_state_handlers(get_current_state());
        let mut first_update = handlers.first_update.borrow_mut();
        if *first_update {
            debug!("calling {}_update.", get_current_state());
            debug!("NOTE: only the first call is logged to avoid noise.");
            *first_update = false;
        }
        (handlers.update)();
    }
}