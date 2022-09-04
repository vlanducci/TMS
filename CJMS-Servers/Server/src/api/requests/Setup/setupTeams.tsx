import { comm_service } from "@cjms_shared/services";
import { TeamModel } from "../../database/models/Team";

export async function setupTeams(team_block:any[]) {
  // var numberOfTeams = team_block[1][1];
  const teams:any[] = team_block.slice(2);
  for (const team of teams) {
    const newTeam = new TeamModel({team_number: team[0], team_name: team[1], affiliation: team[2], ranking: 0});
    await newTeam.save();
  }

  comm_service.senders.sendTeamUpdateEvent('setup');
}