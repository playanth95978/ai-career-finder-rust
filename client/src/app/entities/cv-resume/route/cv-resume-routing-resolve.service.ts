import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { ICvResume } from '../cv-resume.model';
import { CvResumeService } from '../service/cv-resume.service';

const cvResumeResolve = (route: ActivatedRouteSnapshot): Observable<null | ICvResume> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(CvResumeService);
    return service.find(id).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 404) {
          router.navigate(['404']);
        } else {
          router.navigate(['error']);
        }
        return EMPTY;
      }),
    );
  }

  return of(null);
};

export default cvResumeResolve;
