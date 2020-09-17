import { ConnectRunnerAction, RunnerActionType , ConnectedRunnerAction, StartWithRunnerAction, StartedRunnerAction, GotMessageRunnerAction, SentMessageRunnerAction, ConitinueRunnerAction} from "./types";
import { type } from "os";
import { Output, Input, DataType } from "../wsapi/types";

function connect(server:string):ConnectRunnerAction{
    return {type:RunnerActionType.Connect,payload:{server}};
}
function connected(message:string):ConnectedRunnerAction{
    return {type:RunnerActionType.Connected,payload:{message}};
}
function startWith(filter:string):StartWithRunnerAction{
    return {type:RunnerActionType.StartWith,payload:{filter}};
}
function continueWith(name:string,value:string,dataType:DataType):ConitinueRunnerAction{
    return {type:RunnerActionType.Continue,payload:{name,dataType,value}};
}
function started(filter:string):StartedRunnerAction{
    return {type:RunnerActionType.Started,payload:{filter}};
}
function gotMessage(output:Output):GotMessageRunnerAction{
    return {type:RunnerActionType.GotMessage,payload:{output}};
}
function sentMessage(input:Input):SentMessageRunnerAction{
    return {type:RunnerActionType.SentMessage,payload:{input}};
}
const runnerActions = {
    connect,
    connected,
    startWith,
    continueWith,
    started,
    gotMessage,
    sentMessage
}
export default runnerActions;