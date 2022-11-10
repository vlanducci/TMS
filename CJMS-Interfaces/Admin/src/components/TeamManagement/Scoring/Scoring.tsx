import { CJMS_REQUEST_TEAMS } from "@cjms_interfaces/shared";
import { comm_service, initITeam, ITeam, ITeamScore } from "@cjms_shared/services";
import { Button, Grid, MenuItem, Paper, Select, Typography } from "@mui/material";
import { borderColor, color } from "@mui/system";
import { Component } from "react";
import CloudPublish from "./CloudPublish";
import ScoreContainer from "./ScoreContainer";

interface IScoresheet {
  scoresheet:ITeamScore;
  team_number:string;
}

interface IProps {}

interface IState {
  external_teamData:ITeam[];
  scoresheets:IScoresheet[];
}

export default class Scoring extends Component<IProps, IState> {
  constructor(props:any) {
    super(props)

    this.state = {
      external_teamData: [],
      scoresheets: []
    }

    comm_service.listeners.onTeamUpdate(async () => {
      const teamData:ITeam[] = await CJMS_REQUEST_TEAMS(true);
      if (teamData != undefined) {
        this.setTeamData(teamData);
      }
    });
  }

  setTeamData(teams:ITeam[]) {
    this.setState({external_teamData: teams});

    const scores:IScoresheet[] = [];
    for (const team of teams) {
      for (const score of team.scores) {
        scores.push({scoresheet: score, team_number: team.team_number});
      }
    }

    scores.sort((a:IScoresheet, b:IScoresheet) => {
      return new Date(b.scoresheet.updatedAt||0).getTime() - new Date(a.scoresheet.updatedAt||0).getTime();
    });

    this.setState({scoresheets: scores});
  }

  async componentDidMount() {
    const teamData:ITeam[] = await CJMS_REQUEST_TEAMS(true);
    if (teamData != undefined) {
      this.setTeamData(teamData);
    }
  }

  getFilterItems() {
    return (
      <MenuItem>Filter by update timestamp</MenuItem>
    )
  }

  render() {
    return(
      <div className="sc-grid-container">
        <CloudPublish external_teamData={this.state.external_teamData}/>
        <Grid container sx={{backgroundColor: '#18191f', borderRadius: '20px'}}>
          {this.state.scoresheets.map((scoresheet, i) => (
            <ScoreContainer 
              key={i}
              scoresheet={scoresheet}
              team={this.state.external_teamData.find((team) => team.team_number === scoresheet.team_number) || initITeam()}
            />
          ))}
        </Grid>
      </div>
    );
  }
}