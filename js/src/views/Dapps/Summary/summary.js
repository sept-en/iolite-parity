// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import { Container, ContainerTitle, Tags } from '~/ui';

import styles from './summary.css';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const { dappsUrl } = this.context.api;
    const { app } = this.props;

    if (!app) {
      return null;
    }

    const image = this.renderImage(dappsUrl, app);
    const link = this.renderLink(app);

    return (
      <Container className={ styles.container }>
        { image }
        <Tags tags={ [app.type] } />
        <div className={ styles.description }>
          <ContainerTitle
            className={ styles.title }
            title={ link }
            byline={ app.description }
          />
          <div className={ styles.author }>
            { app.author }, v{ app.version }
          </div>
          { this.props.children }
        </div>
      </Container>
    );
  }

  renderImage (dappsUrl, app) {
    if (app.type === 'local') {
      return (
        <img src={ `${dappsUrl}/${app.id}/${app.iconUrl}` } className={ styles.image } />
      );
    }

    return (
      <img src={ `${dappsUrl}${app.image}` } className={ styles.image } />
    );
  }

  renderLink (app) {
    // Special case for web dapp
    if (app.url === 'web') {
      return (
        <Link to={ `/web` }>
          { app.name }
        </Link>
      );
    }

    return (
      <Link to={ `/app/${app.id}` }>
        { app.name }
      </Link>
    );
  }
}
