import { InputType,StartInput, ContinueInput, DataType } from './types';

function start(filter: string): StartInput {
    return { type: InputType.Start, payload: { filter } };
}

function continueWith(name:string, value: string,dataType:DataType): ContinueInput {
    return { type: InputType.Continue, payload: { name,value,dataType } };
}

const apiProto = {
    start,
    continueWith,
};

export default apiProto;
