import { Box, Button, TextField } from '@material-ui/core';
import { createStyles, makeStyles, Theme } from '@material-ui/core/styles';
import React, {ChangeEvent, FormEvent, useState } from 'react';
import runnerActions from './actions';
import { useDispatch } from 'react-redux';

const useStyles = makeStyles((theme: Theme) => createStyles({
    connectButton: {
        marginLeft: theme.spacing(1),
    },
}));

type RunnerScreenState = {
    value:string
};

const RunnerScreen: React.FC = () => {
    const classes = useStyles();
    const [state, setState] = useState<RunnerScreenState>({ value: ''});
    const dispatch = useDispatch();
    const handleValueChange = (event: ChangeEvent<HTMLInputElement>) => {
        const value = event.target.value;
        setState((prevState) => ({
            ...prevState,
            value
        }));
    };
    const handleStartWith = (e: FormEvent) => {
        e.preventDefault();
        dispatch(runnerActions.startWith(state.value));
    };
    return (
        <Box display="flex" flexDirection="column" textAlign="center" flexGrow={1} pt={4}>
            <Box component="form" display="flex" justifyContent="center" alignItems="baseline"
                 mt={2}>
                <TextField label="Filter" value={state.value} onChange={handleValueChange}/>
                <Button className={classes.connectButton} variant="contained" color="primary" onClick={handleStartWith}>
                    Start
                </Button>
            </Box>
        </Box>
    );
};

export default RunnerScreen;
