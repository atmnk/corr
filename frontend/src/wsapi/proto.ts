import { InputType,StartInput, ContinueInput } from './types';

function start(filter: string): StartInput {
    return { type: InputType.Start, payload: { filter } };
}

function continueWith(name:string, value: string): ContinueInput {
    return { type: InputType.Continue, payload: { name,value } };
}

const apiProto = {
    start,
    continueWith,
};

export default apiProto;
