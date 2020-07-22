export enum ApiActionType {
    Write = 'api/write',
    Read = 'api/read',
    Wrote = 'api/wrote'
}

export type WriteApiAction = {
    type: ApiActionType.Write;
    payload: Input;
};
export type WroteApiAction = {
    type: ApiActionType.Wrote;
    payload: Input;
};


export type ReadApiAction = {
    type: ApiActionType.Read;
    payload: Output;
};

export type ApiAction = WriteApiAction | ReadApiAction | WroteApiAction;

export type Input = StartInput | ContinueInput;
export enum InputType {
    Start = 'start',
    Continue = 'continue',
}
export type StartInput = {
    type: InputType.Start;
    payload: { filter: string; };
};
export type ContinueInput = {
    type: InputType.Continue;
    payload: { name:string,value:String,dataType:DataType};
};

export type Output = KnowThatOutput | TellMeOutput | ConnectedOutput | DoneOutput;
export enum OutputType {
    KnowThat = 'knowThat',
    TellMe = 'tellMe',
    Connected = 'connected',
    Done = 'done'
}
export type KnowThatOutput = {
    type: OutputType.KnowThat;
    payload: { message: string; };
};
export type DoneOutput = {
    type: OutputType.Done;
    payload: { message: string; };
};
export type ConnectedOutput = {
    type: OutputType.Connected;
    payload: { message: string; };
};
export type TellMeOutput = {
    type: OutputType.TellMe;
    payload: { name:string,dataType:DataType};
};
export type DataType  = {type : JourneyDataType};
export enum JourneyDataType {
    Long =  'long',
    String = 'string',
    Boolean = 'boolean',
    Double = 'double'
}
    

