use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use super::{GameScoresheet, JudgingScoresheet};

#[derive(JsonSchema, Deserialize, Serialize, Clone)]
pub struct TeamGameScore {
  gp: String,
  referee: String,
  no_show: bool,
  score: u32,
  cloud_published: bool,
  scoresheet: GameScoresheet
}

#[derive(JsonSchema, Deserialize, Serialize, Clone)]
pub struct TeamJudgingScore {
  judge: String,
  no_show: bool,
  score: u32,
  cloud_published: bool,
  scoresheet: JudgingScoresheet
}