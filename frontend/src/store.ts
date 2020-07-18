import { connectRouter, routerMiddleware } from 'connected-react-router';
import { createBrowserHistory } from 'history';
import { applyMiddleware, combineReducers, compose, createStore } from 'redux';
import createSagaMiddleware from 'redux-saga';
import { runnerSaga } from './runner/saga';
import apiSaga from './wsapi/saga';
import { composeWithDevTools } from 'redux-devtools-extension';
export const history = createBrowserHistory();

const rootReducer = combineReducers({
    router: connectRouter(history),
});

export type AppState = ReturnType<typeof rootReducer>;

export default function configureStore(): any {
    const sagaMiddleware = createSagaMiddleware();
    const store = createStore(rootReducer, composeWithDevTools(applyMiddleware(routerMiddleware(history), sagaMiddleware)));

    sagaMiddleware.run(runnerSaga);
    sagaMiddleware.run(apiSaga);
    // sagaMiddleware.run(feedSaga);

    return store;
}
