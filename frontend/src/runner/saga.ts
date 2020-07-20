import { put, StrictEffect, takeEvery } from '@redux-saga/core/effects';
import { RunnerActionType, StartWithRunnerAction, ConitinueRunnerAction } from './types';
import { push } from 'connected-react-router';
import runnerActions from './actions';
import apiActions from '../wsapi/actions';
import apiProto from '../wsapi/proto';
import { ApiActionType, ReadApiAction, InputType, WroteApiAction, OutputType } from '../wsapi/types';

function* handleStartWith(action: StartWithRunnerAction): Generator<StrictEffect> {
    yield put(apiActions.write(apiProto.start(action.payload.filter)));
}
function* handleContinue(action: ConitinueRunnerAction): Generator<StrictEffect> {
    yield put(apiActions.write(apiProto.continueWith(action.payload.name,action.payload.value,action.payload.dataType)));
}
function* handleRead(action: ReadApiAction): Generator<StrictEffect> {
    switch(action.payload.type){
        case OutputType.Connected:
            yield put(runnerActions.connected(action.payload.payload.message));
            yield put(push("/runner"))
            break;
        case OutputType.KnowThat:
            yield put(runnerActions.gotMessage(action.payload))
            break;
        case OutputType.TellMe:
            yield put(runnerActions.gotMessage(action.payload))
            break;
    }
}
function* handleWrote(action: WroteApiAction): Generator<StrictEffect> {
    switch(action.payload.type){
        case InputType.Start:
            yield put(runnerActions.started(action.payload.payload.filter));
            yield put(runnerActions.sentMessage(action.payload))
            break;
        case InputType.Continue:
            yield put(runnerActions.sentMessage(action.payload))
            break;
    }
}

export function* runnerSaga(): Generator<StrictEffect> {
    yield takeEvery(RunnerActionType.StartWith, handleStartWith);
    yield takeEvery(RunnerActionType.Continue, handleContinue);
    yield takeEvery(ApiActionType.Read, handleRead);
    yield takeEvery(ApiActionType.Wrote, handleWrote);
}
