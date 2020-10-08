import { call, fork, put, StrictEffect, takeEvery, take } from '@redux-saga/core/effects';
import { eventChannel, EventChannel } from 'redux-saga';
import apiActions from './actions';
import { ApiActionType, Output, WriteApiAction } from './types';
import {ConnectRunnerAction, RunnerActionType} from '../runner/types';

function createWebSocketChannel(webSocket: WebSocket): EventChannel<Output> {
    return eventChannel<Output>((emit) => {
        webSocket.onmessage = (event): void => {
            const output = JSON.parse(event.data) as Output;
            emit(output);
        };
        return (): void => {
            webSocket.close();
        };
    });
}

function* connectWebSocket(action:ConnectRunnerAction): Generator<StrictEffect> {
    const protocol= window.location.protocol === 'https:' ? 'wss' : 'ws'
    const webSocket = new WebSocket(protocol+"://"+action.payload.server+"/api");
    console.log("Connected to channel");
    const webSocketChannel = (yield call(createWebSocketChannel, webSocket)) as EventChannel<Output>;
    yield fork(read, webSocketChannel);
    yield fork(write, webSocket);
}

function* read(webSocketChannel: EventChannel<Output>): Generator<StrictEffect> {
    while (true) {
        const output = (yield take(webSocketChannel)) as Output;
        yield put(apiActions.read(output));
    }
}

function* write(webSocket: WebSocket): Generator<StrictEffect> {
    while (true) {
        const action = (yield take(ApiActionType.Write)) as WriteApiAction;
        webSocket.send(JSON.stringify(action.payload));
        yield put(apiActions.wrote(action.payload))
    }
}
export default function* apiSaga(): Generator<StrictEffect> {
    yield takeEvery(RunnerActionType.Connect, connectWebSocket);
}
