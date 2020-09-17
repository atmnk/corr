import { Box, Typography } from '@material-ui/core';
import React, { HTMLAttributes } from 'react';
import { Interaction } from './types';
import { InputType, OutputType } from '../wsapi/types';

type InteractionProps = {
    interaction: Interaction;
} & HTMLAttributes<HTMLDivElement>;

const InteractionInfo: React.FC<InteractionProps> = ({ className, interaction }: InteractionProps) => {
    let inner=<div>..loading</div>;
    switch (interaction.type){
        case InputType.Start:
            inner = <div>Starting With Filter {interaction.payload.filter}</div>;
            break;
        case InputType.Continue:
            inner = <div>Provided Value  {interaction.payload.value} for {interaction.payload.name}</div>;
            break;
        case OutputType.KnowThat:
            inner = <div>{interaction.payload.message}</div>;
            break;
        case OutputType.TellMe:
            inner = <div>Please provide value for {interaction.payload.name} of type {interaction.payload.dataType.type} </div>;
            break;
        case OutputType.Done:
            inner = <div>{interaction.payload.message}</div>;
            break;
    }
    return (
    <Box className={className} display="flex" p={1}>
        <Box>
            {/* <Typography variant="body1">{interaction.type}</Typography> */}
            <Typography
                component="span"
                variant="body2"
                color="textSecondary"
            >
                {inner}
            </Typography>
        </Box>
    </Box>
);
    }

export default InteractionInfo;
