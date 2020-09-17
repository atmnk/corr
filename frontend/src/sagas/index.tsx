import { all, call, delay, put, takeEvery } from 'redux-saga/effects'
import { addCount } from '../counter';

export function* incrementAsync() {
  yield delay(1000)
  yield put(addCount(1))
}

export function* watchIncrementAsync() {
  yield takeEvery('INCREMENT_ASYNC', incrementAsync)
}

// single entry point to start all Sagas at once
export default function* rootSaga() {
  yield all([
    call(watchIncrementAsync),
  ])
}
