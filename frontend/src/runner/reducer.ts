import { RunnerState, RunnerAction, RunnerActionType } from "./types";
import { OutputType } from "../wsapi/types";

const initialState: RunnerState = {
    isConnected:false,
    journies:[],
    connectionMessage:null
};
export default function runnerReducer(
    state: RunnerState = initialState,
    action: RunnerAction,
): RunnerState {
    switch (action.type) {
        case RunnerActionType.Connected:
            return {
                ...state,
                isConnected: true,
                connectionMessage: action.payload.message
            };
        case RunnerActionType.GotMessage:
            switch(action.payload.output.type){
                case OutputType.TellMe:
                    return {
                        ...state,
                        journies:[...state.journies.slice(0,-1),{
                            ...state.journies[state.journies.length-1],
                            name:action.payload.output.payload.name,
                            dataType:action.payload.output.payload.dataType,
                            interactions:[...state.journies[state.journies.length-1].interactions,action.payload.output]
                        }]
                    };
                case OutputType.KnowThat:
                    return {
                        ...state,
                        journies:[...state.journies.slice(0,-1),{
                            ...state.journies[state.journies.length-1],
                            interactions:[...state.journies[state.journies.length-1].interactions,action.payload.output]
                        }]
                    };
                case OutputType.Connected:
                    return {
                        ...state,
                        journies:[...state.journies.slice(0,-1),{
                            ...state.journies[state.journies.length-1],
                            interactions:[...state.journies[state.journies.length-1].interactions,action.payload.output]
                        }]
                    };
                case OutputType.Done:

                    return {
                        ...state,
                        journies:[...state.journies.slice(0,-1),{
                            ...state.journies[state.journies.length-1],
                            interactions:[...state.journies[state.journies.length-1].interactions,action.payload.output]
                        },{
                            name:null,
                            dataType:null,
                            interactions:[]
                        }]
                    };
            }
            break
        case RunnerActionType.SentMessage:
            return {
                ...state,
                journies:[...state.journies.slice(0,-1),{
                    ...state.journies[state.journies.length-1],
                    interactions:[...state.journies[state.journies.length-1].interactions,action.payload.input]
                }]
            };
        case RunnerActionType.Started:
            return {
                ...state,
                journies:[...state.journies,{
                    name:null,
                    dataType:null,
                    interactions:[]
                }]
            };
    }
    return state;
}