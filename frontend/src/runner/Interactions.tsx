import { Box } from '@material-ui/core';
import React, { HTMLAttributes } from 'react';
import { Interaction } from './types';
import InteractionInfo from './Interaction';

type InteractionListProps = {
    interactions: Interaction[];
} & HTMLAttributes<HTMLDivElement>;

const InteractionList: React.FC<InteractionListProps> = ({ className, interactions }: InteractionListProps) => (
    <Box className={className} display="flex" flexDirection="column" p={2} width="100%" overflow="auto">
        {interactions.map((interaction) => (
            <InteractionInfo interaction={interaction}/>
        ))}
    </Box>
);

export default InteractionList;
