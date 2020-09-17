import { ApiActionType, Input, Output, ReadApiAction, WriteApiAction, WroteApiAction } from './types';

function write(input: Input): WriteApiAction {
    return { type: ApiActionType.Write, payload: input };
}
function wrote(input: Input): WroteApiAction {
    return { type: ApiActionType.Wrote, payload: input };
}

function read(output: Output): ReadApiAction {
    return { type: ApiActionType.Read, payload: output };
}

const apiActions = {
    write,
    read,
    wrote
};

export default apiActions;