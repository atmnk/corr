import { Box, Button, TextField, Typography } from '@material-ui/core';
import React, {ChangeEvent, FormEvent, useState } from 'react';
import runnerActions from './actions';
import { useDispatch, useSelector } from 'react-redux';
import InteractionList from './Interactions';
import { AppState } from '../store';
import { Redirect, useLocation } from 'react-router-dom';

type RunnerScreenState = {
    value:string
};

const RunnerScreen: React.FC = () => {
    const [state, setState] = useState<RunnerScreenState>({ value: ''});
    const dispatch = useDispatch();
    const {journies} = useSelector((state:AppState)=>state.runner)
    const {name,dataType} = journies.length>0?journies[journies.length-1]:{name:null,dataType:null};
    const interactions = journies.flatMap((j)=>j.interactions);
    const handleValueChange = (event: ChangeEvent<HTMLInputElement>) => {
        const value = event.target.value;
        setState((prevState) => ({
            ...prevState,
            value
        }));
    };
    const handleSend = (e: FormEvent) => {
        e.preventDefault();
        if(name == null){
            dispatch(runnerActions.startWith(state.value));
            
        } else {
            dispatch(runnerActions.continueWith(name!!,state.value,dataType!!));
        }
        
    };
    const location = useLocation();
    const {connectionMessage} = useSelector((state:AppState)=>state.runner)
    const action = name==null? "Start":"Send"
    const placeholder = name==null? "Filter":"Value"
    return !!connectionMessage?
     (
            <Box display="flex" flexDirection="column" flexGrow={1} minHeight={0}>
                <Box>
                {/* <Typography variant="body1">{interaction.type}</Typography> */}
                <Typography
                    component="span"
                    variant="body1"
                    color="textSecondary"
                >
                    {connectionMessage??"You are not connected to server"}
                </Typography>
            </Box>
            <Box display="flex" width="100%">
                <InteractionList interactions={interactions}/>
            </Box>
            <Box component="form" display="flex" justifyContent="center" alignItems="baseline"
                 mt={2}>
                <TextField label={placeholder} value={state.value} onChange={handleValueChange}/>
                <Button variant="contained" color="primary" onClick={handleSend}>
                    {action}
                </Button>
            </Box>
        </Box>
    ):<Redirect to={{ pathname: '/', state: { from: location } }}/>;
};

export default RunnerScreen;
