import { Routes } from '@angular/router';

import { ASC } from 'app/config/navigation.constants';
import { UserRouteAccessService } from 'app/core/auth/user-route-access.service';

import CandidateProfileResolve from './route/candidate-profile-routing-resolve.service';

const candidateProfileRoute: Routes = [
  {
    path: '',
    loadComponent: () => import('./list/candidate-profile').then(m => m.CandidateProfile),
    data: {
      defaultSort: `id,${ASC}`,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/view',
    loadComponent: () => import('./detail/candidate-profile-detail').then(m => m.CandidateProfileDetail),
    resolve: {
      candidateProfile: CandidateProfileResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: 'new',
    loadComponent: () => import('./update/candidate-profile-update').then(m => m.CandidateProfileUpdate),
    resolve: {
      candidateProfile: CandidateProfileResolve,
    },
    canActivate: [UserRouteAccessService],
  },
  {
    path: ':id/edit',
    loadComponent: () => import('./update/candidate-profile-update').then(m => m.CandidateProfileUpdate),
    resolve: {
      candidateProfile: CandidateProfileResolve,
    },
    canActivate: [UserRouteAccessService],
  },
];

export default candidateProfileRoute;
