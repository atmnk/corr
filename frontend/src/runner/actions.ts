import { ConnectRunnerAction, RunnerActionType , ConnectedRunnerAction, StartWithRunnerAction, StartedRunnerAction} from "./types";
import { type } from "os";

function connect(server:string):ConnectRunnerAction{
    return {type:RunnerActionType.Connect,payload:{server}};
}
function connected(to:string):ConnectedRunnerAction{
    return {type:RunnerActionType.Connected,payload:{to}};
}
function startWith(filter:string):StartWithRunnerAction{
    return {type:RunnerActionType.StartWith,payload:{filter}};
}
function started(filter:string):StartedRunnerAction{
    return {type:RunnerActionType.Started,payload:{filter}};
}
const runnerActions = {
    connect,
    connected,
    startWith,
    started
}
export default runnerActions;