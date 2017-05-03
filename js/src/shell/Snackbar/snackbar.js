// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { Snackbar as SnackbarMUI } from 'material-ui';

import { closeSnackbar } from '~/redux/providers/snackbarActions';

const BODY_STYLE = {
  backgroundColor: 'rgba(0, 0, 0, 0.87)',
  borderStyle: 'solid',
  borderColor: '#424242',
  borderWidth: '1px 1px 0 1px'
};

function Snackbar ({ closeSnackbar, cooldown, message, open }) {
  return (
    <SnackbarMUI
      autoHideDuration={ cooldown }
      bodyStyle={ BODY_STYLE }
      message={ message }
      open={ open }
      onRequestClose={ closeSnackbar }
    />
  );
}

Snackbar.propTypes = {
  closeSnackbar: PropTypes.func.isRequired,
  cooldown: PropTypes.number,
  message: PropTypes.any,
  open: PropTypes.bool
};

function mapStateToProps (state) {
  const { open, message, cooldown } = state.snackbar;

  return { open, message, cooldown };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    closeSnackbar
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Snackbar);
