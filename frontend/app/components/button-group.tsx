import React from 'react';

type Props = {
    children: React.ReactNode;
}
const ButtonGroup = ({children}: Props) => {
    return (
        <div>
            {children}
        </div>
    );
};

export default ButtonGroup;
