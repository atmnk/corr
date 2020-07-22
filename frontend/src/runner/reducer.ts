import { RunnerState, RunnerAction, RunnerActionType } from "./types";
import { OutputType } from "../wsapi/types";

const initialState: RunnerState = {
    isConnected:false,
    journey:null,
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
                        journey: {
                            name:action.payload.output.payload.name,
                            dataType:action.payload.output.payload.dataType,
                            interactions:[...state.journey!!.interactions,action.payload.output]
                        }
                    };
                case OutputType.KnowThat:
                    return {
                        ...state,
                        journey: {
                            ...state.journey!!,
                            interactions:[...state.journey!!.interactions,action.payload.output]
                        }
                    };
                case OutputType.Connected:
                    return {
                        ...state,
                        journey: {
                            ...state.journey!!,
                            interactions:[...state.journey!!.interactions,action.payload.output]
                        }
                    };
                case OutputType.Done:
                    return {
                        ...state,
                        journey: {
                            ...state.journey!!,
                            interactions:[...state.journey!!.interactions,action.payload.output]
                        }
                    };
            }            
        case RunnerActionType.SentMessage:
            return {
                ...state,
                journey: {
                    ...state.journey!!,
                    interactions:[...state.journey!!.interactions,action.payload.input]
                }
            };
        case RunnerActionType.Started:
            return {
                ...state,
                journey: {
                    name:null,
                    dataType:null,
                    interactions:[]
                }
            };
    }
    return state;
}