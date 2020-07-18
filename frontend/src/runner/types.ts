export enum RunnerActionType {
    Connect = 'runner/connect',
    Connected = 'runner/connected',
    StartWith = 'runner/startWith',
    Started = 'runner/started'
}
export type ConnectRunnerAction = {
    type:RunnerActionType.Connect,
    payload: {server:string}
}
export type ConnectedRunnerAction = {
    type:RunnerActionType.Connected,
    payload: {to:string}
}
export type StartWithRunnerAction = {
    type:RunnerActionType.StartWith,
    payload: {filter:string}
}
export type StartedRunnerAction = {
    type:RunnerActionType.Started,
    payload: {filter:string}
}