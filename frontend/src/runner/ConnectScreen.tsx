import { Box, Button, TextField, Typography } from '@material-ui/core';
import { createStyles, makeStyles, Theme } from '@material-ui/core/styles';
import React, {ChangeEvent, FormEvent, useState } from 'react';
import runnerActions from './actions';
import { useDispatch } from 'react-redux';

const useStyles = makeStyles((theme: Theme) => createStyles({
    connectButton: {
        marginLeft: theme.spacing(1),
    },
}));

type ConnectScreenState = {
    server:string
};

const ConnectScreen: React.FC = () => {
    const classes = useStyles();
    const [state, setState] = useState<ConnectScreenState>({ server: ''});
    const dispatch = useDispatch();
    const handleServerChange = (event: ChangeEvent<HTMLInputElement>) => {
        const server = event.target.value;
        setState((prevState) => ({
            ...prevState,
            server
        }));
    };
    const handleConnect = (e: FormEvent) => {
        e.preventDefault();
        const server = state.server.trim();
        dispatch(runnerActions.connect(server));
    };
    return (
        <Box display="flex" flexDirection="column" textAlign="center" flexGrow={1} pt={4}>
            <Typography variant="h3">
                Welcome!
            </Typography>
            <Box component="form" display="flex" justifyContent="center" alignItems="baseline"
                 mt={2}>
                <TextField label="Server" value={state.server} onChange={handleServerChange}/>
                <Button className={classes.connectButton} variant="contained" color="primary" onClick={handleConnect}>
                    Connect
                </Button>
            </Box>
        </Box>
    );
};

export default ConnectScreen;
