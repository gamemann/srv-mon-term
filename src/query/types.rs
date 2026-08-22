pub mod a2s;
pub mod error;
pub mod ext;
pub mod fivem;
pub mod gamespy;
pub mod minecraft;
pub mod port;
pub mod quake3;

use crate::query::types::{
    a2s::QueryA2sCtx,
    fivem::QueryFiveMCtx,
    gamespy::{QueryGameSpy1Ctx, QueryGameSpy3Ctx},
    minecraft::{QueryBedrockCtx, QueryMinecraftCtx},
    quake3::QueryQuake3Ctx,
};

pub enum Query {
    A2s(QueryA2sCtx),
    Quake3(QueryQuake3Ctx),
    Minecraft(QueryMinecraftCtx),
    Bedrock(QueryBedrockCtx),
    GameSpy1(QueryGameSpy1Ctx),
    GameSpy3(QueryGameSpy3Ctx),
    FiveM(QueryFiveMCtx),
}
