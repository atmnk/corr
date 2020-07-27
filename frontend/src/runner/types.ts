import { Input, Output, DataType } from "../wsapi/types"

export enum RunnerActionType {
    Connect = 'runner/connect',
    Connected = 'runner/connected',
    Continue = 'runner/continue',
    StartWith = 'runner/startWith',
    Started = 'runner/started',
    GotMessage = 'runner/gotMessage',
    SentMessage = 'runner/sentMessage'
}
export type ConnectRunnerAction = {
    type:RunnerActionType.Connect,
    payload: {server:string}
}
export type ConnectedRunnerAction = {
    type:RunnerActionType.Connected,
    payload: {message:string}
}
export type StartWithRunnerAction = {
    type:RunnerActionType.StartWith,
    payload: {filter:string}
}
export type ConitinueRunnerAction = {
    type:RunnerActionType.Continue,
    payload: {name:string,value:string,dataType:DataType}
}
export type StartedRunnerAction = {
    type:RunnerActionType.Started,
    payload: {filter:string}
}
export type GotMessageRunnerAction = {
    type:RunnerActionType.GotMessage,
    payload: {output:Output}
}
export type SentMessageRunnerAction = {
    type:RunnerActionType.SentMessage,
    payload: {input:Input}
}
export type Interaction = Input | Output
export type Journey = {
    name:string | null,
    dataType: DataType | null,
    interactions: Interaction[]
}
export type RunnerState = {
    isConnected:boolean,
    connectionMessage:string | null,
    journies:Journey[]
}
export type RunnerAction = 
ConnectRunnerAction
| ConnectedRunnerAction
| StartWithRunnerAction
| StartedRunnerAction
| GotMessageRunnerAction
| SentMessageRunnerAction