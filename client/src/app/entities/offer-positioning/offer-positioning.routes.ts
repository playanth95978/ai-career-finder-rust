import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import OfferPositioningResolve from './route/offer-positioning-routing-resolve.service';

const offerPositioningRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/offer-positioning').then(m => m.OfferPositioning),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/offer-positioning-detail').then(m => m.OfferPositioningDetail),
    resolve: {
      offerPositioning: OfferPositioningResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/offer-positioning-update').then(m => m.OfferPositioningUpdate),
    resolve: {
      offerPositioning: OfferPositioningResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/offer-positioning-update').then(m => m.OfferPositioningUpdate),
    resolve: {
      offerPositioning: OfferPositioningResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default offerPositioningRoute;
