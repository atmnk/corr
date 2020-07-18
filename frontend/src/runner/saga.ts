import { put, StrictEffect, takeEvery } from '@redux-saga/core/effects';
import { ConnectRunnerAction, RunnerActionType, StartWithRunnerAction } from './types';
import { push } from 'connected-react-router';
import runnerActions from './actions';
import apiActions from '../wsapi/actions';
import apiProto from '../wsapi/proto';

function* handleConnect(action: ConnectRunnerAction): Generator<StrictEffect> {
    yield put(runnerActions.connected(action.payload.server));
    yield put(push('/runner'));
}
function* handleStartWith(action: StartWithRunnerAction): Generator<StrictEffect> {
    yield put(apiActions.write(apiProto.start(action.payload.filter)));
    yield put(runnerActions.started(action.payload.filter));
}

export function* runnerSaga(): Generator<StrictEffect> {
    yield takeEvery(RunnerActionType.Connect, handleConnect);
    yield takeEvery(RunnerActionType.StartWith, handleStartWith);
}
