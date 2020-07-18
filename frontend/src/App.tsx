import React from 'react';
import { Route, Switch } from 'react-router-dom';
import { makeStyles } from '@material-ui/core/styles';
import ConnectScreen from './runner/ConnectScreen';
import RunnerScreen from './runner/RunnerScreen';
const useStyles = makeStyles({
  '@global': {
      html: {
          height: '100%',
      },
      body: {
          height: '100%',
      },
      '#root': {
          height: '100%',
          display: 'flex',
          flexDirection: 'column',
      },
  },
});
export const incrementAsync = () => ({
  type: 'INCREMENT_ASYNC',
});
const App: React.FC = () => {
  useStyles();
  return (
      <>
          <Switch>
              <Route exact path="/" component={ConnectScreen}/>
              <Route exact path="/runner" component={RunnerScreen}/>
          </Switch>
      </>
  );
};

export default App;
