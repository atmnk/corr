export enum ApiActionType {
    Write = 'api/write',
    Read = 'api/read',
}

export type WriteApiAction = {
    type: ApiActionType.Write;
    payload: Input;
};


export type ReadApiAction = {
    type: ApiActionType.Read;
    payload: Output;
};



export type ApiAction = WriteApiAction | ReadApiAction;

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
    payload: { name:string,value:String};
};

export type Output = KnowThatOutput | TellMeOutput;
export enum OutputType {
    KnowThat = 'knowThat',
    TellMe = 'tellMe',
}
export type KnowThatOutput = {
    type: OutputType.KnowThat;
    payload: { phrase: string; };
};
export type TellMeOutput = {
    type: OutputType.TellMe;
    payload: { name:string,data_type:String};
};

